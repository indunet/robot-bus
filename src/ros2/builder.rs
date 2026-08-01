//! Chained builder for [`Ros2Bridge`].

use std::any::Any;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use prost::Message as ProstMessage;
use rclrs::{
    Context, CreateBasicExecutor, DynamicPublisher, DynamicSubscription, MessageTypeName,
    SpinOptions,
};

use crate::discovery::DiscoverOpts;
use crate::errors::{BusError, Result};
use crate::runtime::{Node, NodeOptions, ServiceHandler, TopicPublisherRaw};
use crate::sensor_msgs::msg::v1::Imu as BusImu;
use crate::std_msgs::msg::v1::String as BusString;
use crate::std_srvs::srv::v1::{
    SetBool as BusSetBool, SetBoolRequest as BusSetBoolRequest,
    SetBoolResponse as BusSetBoolResponse, Trigger as BusTrigger,
    TriggerRequest as BusTriggerRequest, TriggerResponse as BusTriggerResponse,
};

use super::convert;
use super::echo::EchoFilter;
use super::vendor::std_srvs::srv as ros_srv;
use super::yaml;

/// Default timeout for bridged service calls (ROS↔bus).
pub const SERVICE_CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// Topic / service bridge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    RosToBus,
    BusToRos,
    Both,
}

/// Whitelisted ROS message kinds (topics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgKind {
    String,
    Imu,
}

impl MsgKind {
    pub fn type_name(self) -> &'static str {
        match self {
            Self::String => "std_msgs/msg/String",
            Self::Imu => "sensor_msgs/msg/Imu",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "std_msgs/msg/String" => Ok(Self::String),
            "sensor_msgs/msg/Imu" => Ok(Self::Imu),
            other => Err(BusError::Protocol(format!(
                "unsupported ros2 bridge type {other:?}; supported: std_msgs/msg/String, sensor_msgs/msg/Imu"
            ))),
        }
    }
}

/// Whitelisted ROS service kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrvKind {
    Trigger,
    SetBool,
}

impl SrvKind {
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Trigger => "std_srvs/srv/Trigger",
            Self::SetBool => "std_srvs/srv/SetBool",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "std_srvs/srv/Trigger" => Ok(Self::Trigger),
            "std_srvs/srv/SetBool" => Ok(Self::SetBool),
            other => Err(BusError::Protocol(format!(
                "unsupported ros2 bridge service type {other:?}; supported: std_srvs/srv/Trigger, std_srvs/srv/SetBool"
            ))),
        }
    }
}

#[cfg(test)]
mod srv_kind_tests {
    use super::*;

    #[test]
    fn parse_trigger_set_bool() {
        assert_eq!(SrvKind::parse("std_srvs/srv/Trigger").unwrap(), SrvKind::Trigger);
        assert_eq!(SrvKind::parse("std_srvs/srv/SetBool").unwrap(), SrvKind::SetBool);
        assert!(SrvKind::parse("std_srvs/srv/Empty").is_err());
    }

    #[test]
    fn reject_both_for_services() {
        let res = Ros2Bridge::new("t").add_service("/a", "/a", SrvKind::Trigger, Direction::Both);
        assert!(res.is_err());
        let msg = res.err().unwrap().to_string();
        assert!(msg.contains("both"));
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RouteSpec {
    ros_topic: String,
    bus_topic: String,
    kind: MsgKind,
    direction: Direction,
}

#[derive(Debug, Clone)]
pub(crate) struct ServiceRouteSpec {
    ros_service: String,
    bus_service: String,
    kind: SrvKind,
    direction: Direction,
}

/// Fluent builder: `Ros2Bridge::new(...).bus_tcp(...).route(...).string().add().build()?`.
pub struct Ros2BridgeBuilder {
    name: String,
    bus_options: NodeOptions,
    routes: Vec<RouteSpec>,
    services: Vec<ServiceRouteSpec>,
}

/// In-process dual-stack bridge (ROS 2 + robot-bus).
pub struct Ros2Bridge {
    bus_node: Node,
    /// ROS→bus payloads (ZMQ publisher is not `Send`; drain on the spin thread).
    ros_to_bus_rx: Receiver<(String, Vec<u8>)>,
    bus_pubs: std::collections::HashMap<String, TopicPublisherRaw>,
    _echo: Arc<Mutex<EchoFilter>>,
    _ros_subs: Vec<DynamicSubscription>,
    _ros_pubs: Vec<DynamicPublisher>,
    /// Keeps typed ROS services / clients alive for the bridge lifetime.
    _ros_entities: Vec<Box<dyn Any + Send + Sync>>,
    ros_halt: Arc<AtomicBool>,
    _ros_spin: Option<JoinHandle<()>>,
}

impl Drop for Ros2Bridge {
    fn drop(&mut self) {
        self.ros_halt.store(true, Ordering::SeqCst);
        if let Some(h) = self._ros_spin.take() {
            let _ = h.join();
        }
    }
}

/// Intermediate topic route configuration before [`RouteBuilder::add`].
pub struct RouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_topic: String,
    bus_topic: String,
    kind: Option<MsgKind>,
    direction: Direction,
}

/// Intermediate service route configuration before [`ServiceRouteBuilder::add`].
pub struct ServiceRouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_service: String,
    bus_service: String,
    kind: Option<SrvKind>,
    direction: Direction,
}

impl Ros2Bridge {
    pub fn new(name: impl Into<String>) -> Ros2BridgeBuilder {
        Ros2BridgeBuilder {
            name: name.into(),
            bus_options: NodeOptions::tcp(),
            routes: Vec::new(),
            services: Vec::new(),
        }
    }

    pub fn from_yaml(path: impl AsRef<Path>) -> Result<Self> {
        yaml::builder_from_yaml(path)?.build()
    }

    pub fn spin(&mut self) -> Result<()> {
        loop {
            self.spin_once(Duration::from_millis(10))?;
        }
    }

    pub fn spin_once(&mut self, timeout: Duration) -> Result<()> {
        // ROS executor runs on a background thread so Bus→ROS service handlers
        // can wait on client Promises without nested-spin deadlocks.
        while let Ok((topic, payload)) = self.ros_to_bus_rx.try_recv() {
            if let Some(pub_) = self.bus_pubs.get(&topic) {
                if let Err(e) = pub_.publish(&payload) {
                    log::warn!("ros→bus publish {topic}: {e}");
                }
            }
        }
        self.bus_node.spin_once(Some(timeout))?;
        Ok(())
    }
}

impl Ros2BridgeBuilder {
    pub fn bus_tcp(mut self, host: impl Into<String>) -> Self {
        self.bus_options = NodeOptions::tcp_at(host);
        self
    }

    pub fn bus_ipc(mut self) -> Self {
        self.bus_options = NodeOptions::ipc();
        self
    }

    pub fn bus_ipc_at(mut self, dir: impl AsRef<str>) -> Self {
        self.bus_options = NodeOptions::ipc_at(dir);
        self
    }

    /// Discover a broker (UDP), then connect over TCP.
    pub fn bus_discover(self, domain_id: u32) -> Result<Self> {
        self.bus_discover_ex(domain_id, None, None)
    }

    /// Discover with optional timeout (seconds) and broker id filter.
    pub fn bus_discover_ex(
        mut self,
        domain_id: u32,
        timeout_secs: Option<f64>,
        broker_id: Option<String>,
    ) -> Result<Self> {
        let mut opts = DiscoverOpts {
            domain_id,
            ..Default::default()
        };
        if let Some(t) = timeout_secs {
            opts.timeout = Duration::from_secs_f64(t);
        }
        opts.broker_id = broker_id;
        self.bus_options = NodeOptions::tcp().discover(opts)?;
        Ok(self)
    }

    pub fn route(
        self,
        ros_topic: impl Into<String>,
        bus_topic: impl Into<String>,
    ) -> RouteBuilder {
        RouteBuilder {
            parent: self,
            ros_topic: ros_topic.into(),
            bus_topic: bus_topic.into(),
            kind: None,
            direction: Direction::Both,
        }
    }

    pub fn service(
        self,
        ros_service: impl Into<String>,
        bus_service: impl Into<String>,
    ) -> ServiceRouteBuilder {
        ServiceRouteBuilder {
            parent: self,
            ros_service: ros_service.into(),
            bus_service: bus_service.into(),
            kind: None,
            direction: Direction::RosToBus,
        }
    }

    pub(crate) fn push_route(
        mut self,
        ros_topic: String,
        bus_topic: String,
        kind: MsgKind,
        direction: Direction,
    ) -> Self {
        self.routes.push(RouteSpec {
            ros_topic,
            bus_topic,
            kind,
            direction,
        });
        self
    }

    pub(crate) fn push_service(
        mut self,
        ros_service: String,
        bus_service: String,
        kind: SrvKind,
        direction: Direction,
    ) -> Result<Self> {
        if matches!(direction, Direction::Both) {
            return Err(BusError::Protocol(
                "ros2 bridge services do not support direction=both; use ros_to_bus or bus_to_ros"
                    .into(),
            ));
        }
        self.services.push(ServiceRouteSpec {
            ros_service,
            bus_service,
            kind,
            direction,
        });
        Ok(self)
    }

    /// Add a typed topic route (for FFI / non-fluent callers).
    pub fn add_route(
        self,
        ros_topic: impl Into<String>,
        bus_topic: impl Into<String>,
        kind: MsgKind,
        direction: Direction,
    ) -> Self {
        self.push_route(ros_topic.into(), bus_topic.into(), kind, direction)
    }

    /// Add a typed service route (for FFI / non-fluent callers).
    pub fn add_service(
        self,
        ros_service: impl Into<String>,
        bus_service: impl Into<String>,
        kind: SrvKind,
        direction: Direction,
    ) -> Result<Self> {
        self.push_service(
            ros_service.into(),
            bus_service.into(),
            kind,
            direction,
        )
    }

    pub fn build(self) -> Result<Ros2Bridge> {
        if self.routes.is_empty() && self.services.is_empty() {
            return Err(BusError::Protocol(
                "Ros2Bridge requires at least one topic route or service".into(),
            ));
        }

        let context = Context::default_from_env().map_err(|e| {
            BusError::Protocol(format!(
                "rclrs Context::default_from_env failed ({e}); source ROS 2 first"
            ))
        })?;
        let mut ros_executor = context.create_basic_executor();
        let ros_node = ros_executor
            .create_node(self.name.as_str())
            .map_err(|e| BusError::Protocol(format!("rclrs create_node: {e}")))?;

        let mut bus_node = Node::with_options(format!("{}_bus", self.name), self.bus_options);
        let echo = Arc::new(Mutex::new(EchoFilter::new(Duration::from_millis(500))));
        let (ros_to_bus_tx, ros_to_bus_rx) = mpsc::sync_channel::<(String, Vec<u8>)>(1024);

        let mut ros_subs = Vec::new();
        let mut ros_pubs = Vec::new();
        let mut bus_pubs = std::collections::HashMap::new();
        let mut ros_entities: Vec<Box<dyn Any + Send + Sync>> = Vec::new();

        for route in &self.routes {
            wire_route(
                &ros_node,
                &mut bus_node,
                &echo,
                &ros_to_bus_tx,
                &mut bus_pubs,
                route,
                &mut ros_subs,
                &mut ros_pubs,
            )?;
        }

        for svc in &self.services {
            wire_service_route(&ros_node, &mut bus_node, svc, &mut ros_entities)?;
        }

        let ros_halt = Arc::new(AtomicBool::new(false));
        let halt_flag = Arc::clone(&ros_halt);
        let ros_spin = thread::Builder::new()
            .name("ros2_bridge_spin".into())
            .spawn(move || {
                while !halt_flag.load(Ordering::Relaxed) {
                    let _ = ros_executor.spin(
                        SpinOptions::spin_once().timeout(Duration::from_millis(10)),
                    );
                }
            })
            .map_err(|e| BusError::Protocol(format!("spawn ros2 spin thread: {e}")))?;

        Ok(Ros2Bridge {
            bus_node,
            ros_to_bus_rx,
            bus_pubs,
            _echo: echo,
            _ros_subs: ros_subs,
            _ros_pubs: ros_pubs,
            _ros_entities: ros_entities,
            ros_halt,
            _ros_spin: Some(ros_spin),
        })
    }
}

impl RouteBuilder {
    pub fn string(mut self) -> Self {
        self.kind = Some(MsgKind::String);
        self
    }

    pub fn imu(mut self) -> Self {
        self.kind = Some(MsgKind::Imu);
        self
    }

    pub fn msg_kind(mut self, kind: MsgKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn add(self) -> Ros2BridgeBuilder {
        let kind = self.kind.unwrap_or_else(|| {
            panic!("ros2 bridge route: call .string() or .imu() before .add()")
        });
        self.parent
            .push_route(self.ros_topic, self.bus_topic, kind, self.direction)
    }
}

impl ServiceRouteBuilder {
    pub fn trigger(mut self) -> Self {
        self.kind = Some(SrvKind::Trigger);
        self
    }

    pub fn set_bool(mut self) -> Self {
        self.kind = Some(SrvKind::SetBool);
        self
    }

    pub fn srv_kind(mut self, kind: SrvKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        let kind = self.kind.ok_or_else(|| {
            BusError::Protocol(
                "ros2 bridge service: call .trigger() or .set_bool() before .add()".into(),
            )
        })?;
        self.parent
            .push_service(self.ros_service, self.bus_service, kind, self.direction)
    }
}

fn wire_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    echo: &Arc<Mutex<EchoFilter>>,
    ros_to_bus_tx: &SyncSender<(String, Vec<u8>)>,
    bus_pubs: &mut std::collections::HashMap<String, TopicPublisherRaw>,
    route: &RouteSpec,
    ros_subs: &mut Vec<DynamicSubscription>,
    ros_pubs: &mut Vec<DynamicPublisher>,
) -> Result<()> {
    let type_name: MessageTypeName = convert::type_name(route.kind);
    let ros_topic = route.ros_topic.clone();
    let bus_topic = route.bus_topic.clone();

    let ros_pub = if matches!(route.direction, Direction::BusToRos | Direction::Both) {
        let p = ros_node
            .create_dynamic_publisher(type_name.clone(), ros_topic.as_str())
            .map_err(|e| BusError::Protocol(format!("ros dynamic publisher: {e}")))?;
        ros_pubs.push(p.clone());
        Some(p)
    } else {
        None
    };

    if matches!(route.direction, Direction::RosToBus | Direction::Both) {
        bus_pubs.insert(
            bus_topic.clone(),
            bus_node.create_publisher_raw(bus_topic.as_str())?,
        );
        let echo = Arc::clone(echo);
        let kind = route.kind;
        let tx = ros_to_bus_tx.clone();
        let bus_topic_cb = bus_topic.clone();
        let sub = ros_node
            .create_dynamic_subscription(
                type_name,
                ros_topic.as_str(),
                move |dyn_msg, _info| {
                    let payload = match kind {
                        MsgKind::String => match convert::string_dyn_to_bus(&dyn_msg) {
                            Ok(m) => m.encode_to_vec(),
                            Err(e) => {
                                log::warn!("ros→bus String convert: {e}");
                                return;
                            }
                        },
                        MsgKind::Imu => match convert::imu_dyn_to_bus(&dyn_msg) {
                            Ok(m) => m.encode_to_vec(),
                            Err(e) => {
                                log::warn!("ros→bus Imu convert: {e}");
                                return;
                            }
                        },
                    };
                    if let Ok(mut g) = echo.lock() {
                        if g.is_echo(&payload) {
                            return;
                        }
                        g.remember(&payload);
                    }
                    if let Err(e) = tx.send((bus_topic_cb.clone(), payload)) {
                        log::warn!("ros→bus channel: {e}");
                    }
                },
            )
            .map_err(|e| BusError::Protocol(format!("ros dynamic subscription: {e}")))?;
        ros_subs.push(sub);
    }

    if let Some(ros_pub) = ros_pub {
        let echo = Arc::clone(echo);
        match route.kind {
            MsgKind::String => {
                bus_node.create_subscription::<BusString, _>(
                    bus_topic.as_str(),
                    move |_topic, msg| {
                        let payload = msg.encode_to_vec();
                        if let Ok(mut g) = echo.lock() {
                            if g.is_echo(&payload) {
                                return;
                            }
                            g.remember(&payload);
                        }
                        match convert::string_bus_to_dyn(&msg) {
                            Ok(dyn_msg) => {
                                if let Err(e) = ros_pub.publish(dyn_msg) {
                                    log::warn!("bus→ros String publish: {e}");
                                }
                            }
                            Err(e) => log::warn!("bus→ros String convert: {e}"),
                        }
                    },
                    None,
                )?;
            }
            MsgKind::Imu => {
                bus_node.create_subscription::<BusImu, _>(
                    bus_topic.as_str(),
                    move |_topic, msg| {
                        let payload = msg.encode_to_vec();
                        if let Ok(mut g) = echo.lock() {
                            if g.is_echo(&payload) {
                                return;
                            }
                            g.remember(&payload);
                        }
                        match convert::imu_bus_to_dyn(&msg) {
                            Ok(dyn_msg) => {
                                if let Err(e) = ros_pub.publish(dyn_msg) {
                                    log::warn!("bus→ros Imu publish: {e}");
                                }
                            }
                            Err(e) => log::warn!("bus→ros Imu convert: {e}"),
                        }
                    },
                    None,
                )?;
            }
        }
    }

    Ok(())
}

fn wire_service_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    route: &ServiceRouteSpec,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    match (route.kind, route.direction) {
        (SrvKind::Trigger, Direction::RosToBus) => {
            let bus_client = Arc::new(Mutex::new(
                bus_node.create_client::<BusTrigger>(route.bus_service.as_str())?,
            ));
            let timeout = SERVICE_CALL_TIMEOUT;
            let srv = ros_node
                .create_service::<ros_srv::Trigger, _>(
                    route.ros_service.as_str(),
                    move |_req: ros_srv::Trigger_Request| {
                        let bus_req = convert::trigger_ros_req_to_bus(&_req);
                        let guard = match bus_client.lock() {
                            Ok(g) => g,
                            Err(e) => {
                                return ros_srv::Trigger_Response {
                                    success: false,
                                    message: format!("bus client lock poisoned: {e}"),
                                };
                            }
                        };
                        match guard.call(&bus_req, Some(timeout)) {
                            Ok(bus_resp) => convert::trigger_bus_resp_to_ros(&bus_resp),
                            Err(e) => ros_srv::Trigger_Response {
                                success: false,
                                message: format!("bus call failed: {e}"),
                            },
                        }
                    },
                )
                .map_err(|e| BusError::Protocol(format!("ros create_service Trigger: {e}")))?;
            ros_entities.push(Box::new(srv));
        }
        (SrvKind::SetBool, Direction::RosToBus) => {
            let bus_client = Arc::new(Mutex::new(
                bus_node.create_client::<BusSetBool>(route.bus_service.as_str())?,
            ));
            let timeout = SERVICE_CALL_TIMEOUT;
            let srv = ros_node
                .create_service::<ros_srv::SetBool, _>(
                    route.ros_service.as_str(),
                    move |req: ros_srv::SetBool_Request| {
                        let bus_req = convert::set_bool_ros_req_to_bus(&req);
                        let guard = match bus_client.lock() {
                            Ok(g) => g,
                            Err(e) => {
                                return ros_srv::SetBool_Response {
                                    success: false,
                                    message: format!("bus client lock poisoned: {e}"),
                                };
                            }
                        };
                        match guard.call(&bus_req, Some(timeout)) {
                            Ok(bus_resp) => convert::set_bool_bus_resp_to_ros(&bus_resp),
                            Err(e) => ros_srv::SetBool_Response {
                                success: false,
                                message: format!("bus call failed: {e}"),
                            },
                        }
                    },
                )
                .map_err(|e| BusError::Protocol(format!("ros create_service SetBool: {e}")))?;
            ros_entities.push(Box::new(srv));
        }
        (SrvKind::Trigger, Direction::BusToRos) => {
            let ros_client = ros_node
                .create_client::<ros_srv::Trigger>(route.ros_service.as_str())
                .map_err(|e| BusError::Protocol(format!("ros create_client Trigger: {e}")))?;
            ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = SERVICE_CALL_TIMEOUT;
            let handler: ServiceHandler = Arc::new(move |body| {
                let bus_req = match BusTriggerRequest::decode(body) {
                    Ok(r) => r,
                    Err(e) => {
                        return BusTriggerResponse {
                            success: false,
                            message: format!("decode TriggerRequest: {e}"),
                        }
                        .encode_to_vec();
                    }
                };
                match call_ros_trigger(&ros_client, &bus_req, timeout) {
                    Ok(resp) => resp.encode_to_vec(),
                    Err(msg) => BusTriggerResponse {
                        success: false,
                        message: msg,
                    }
                    .encode_to_vec(),
                }
            });
            let _ = bus_node.create_service_raw(route.bus_service.as_str(), handler, None)?;
        }
        (SrvKind::SetBool, Direction::BusToRos) => {
            let ros_client = ros_node
                .create_client::<ros_srv::SetBool>(route.ros_service.as_str())
                .map_err(|e| BusError::Protocol(format!("ros create_client SetBool: {e}")))?;
            ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = SERVICE_CALL_TIMEOUT;
            let handler: ServiceHandler = Arc::new(move |body| {
                let bus_req = match BusSetBoolRequest::decode(body) {
                    Ok(r) => r,
                    Err(e) => {
                        return BusSetBoolResponse {
                            success: false,
                            message: format!("decode SetBoolRequest: {e}"),
                        }
                        .encode_to_vec();
                    }
                };
                match call_ros_set_bool(&ros_client, &bus_req, timeout) {
                    Ok(resp) => resp.encode_to_vec(),
                    Err(msg) => BusSetBoolResponse {
                        success: false,
                        message: msg,
                    }
                    .encode_to_vec(),
                }
            });
            let _ = bus_node.create_service_raw(route.bus_service.as_str(), handler, None)?;
        }
        (_, Direction::Both) => {
            return Err(BusError::Protocol(
                "ros2 bridge services do not support direction=both".into(),
            ));
        }
    }
    Ok(())
}

fn wait_service_ready(client_ready: impl Fn() -> bool, timeout: Duration) -> std::result::Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if client_ready() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("timed out waiting for ROS service".into())
}

fn call_ros_trigger(
    client: &rclrs::Client<ros_srv::Trigger>,
    bus_req: &BusTriggerRequest,
    timeout: Duration,
) -> std::result::Result<BusTriggerResponse, String> {
    wait_service_ready(|| client.service_is_ready().unwrap_or(false), timeout)?;
    let ros_req = convert::trigger_bus_req_to_ros(bus_req);
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: ros_srv::Trigger_Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros Trigger call: {e}"))?;
    match rx.recv_timeout(timeout) {
        Ok(resp) => Ok(convert::trigger_ros_resp_to_bus(&resp)),
        Err(_) => Err("timed out waiting for ROS Trigger response".into()),
    }
}

fn call_ros_set_bool(
    client: &rclrs::Client<ros_srv::SetBool>,
    bus_req: &BusSetBoolRequest,
    timeout: Duration,
) -> std::result::Result<BusSetBoolResponse, String> {
    wait_service_ready(|| client.service_is_ready().unwrap_or(false), timeout)?;
    let ros_req = convert::set_bool_bus_req_to_ros(bus_req);
    let (tx, rx) = mpsc::sync_channel(1);
    let _promise = client
        .call_then(ros_req, move |resp: ros_srv::SetBool_Response| {
            let _ = tx.send(resp);
        })
        .map_err(|e| format!("ros SetBool call: {e}"))?;
    match rx.recv_timeout(timeout) {
        Ok(resp) => Ok(convert::set_bool_ros_resp_to_bus(&resp)),
        Err(_) => Err("timed out waiting for ROS SetBool response".into()),
    }
}
