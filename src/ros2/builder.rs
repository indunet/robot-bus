//! Chained builder for [`Ros2Bridge`].

use std::any::Any;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use prost::Message as ProstMessage;
use rclrs::vendor::example_interfaces::action as ros_act;
use rclrs::{
    BeginAcceptedGoal, Context as RosContext, CreateBasicExecutor, DynamicPublisher,
    DynamicSubscription, GoalClient, SpinOptions,
};

use crate::action::v1::{
    Fibonacci as BusFibonacci, FibonacciGoal as BusFibonacciGoal,
    FibonacciResult as BusFibonacciResult,
};
use crate::discovery::DiscoverOpts;
use crate::errors::{BusError, Result};
use crate::runtime::{
    ActionGoalHandler, MessageCallback, Node, NodeOptions, ServiceHandler, TopicPublisherRaw,
};
use crate::std_srvs::srv::v1::{
    SetBool as BusSetBool, SetBoolRequest as BusSetBoolRequest,
    SetBoolResponse as BusSetBoolResponse, Trigger as BusTrigger,
    TriggerRequest as BusTriggerRequest, TriggerResponse as BusTriggerResponse,
};

use super::codec::{self, TopicCodec};
use super::convert;
use super::echo::EchoFilter;
use super::vendor::std_srvs::srv as ros_srv;
use super::yaml;

/// Default timeout for bridged service calls (ROS↔bus).
pub const SERVICE_CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for bridged action goals (ROS↔bus).
pub const ACTION_CALL_TIMEOUT: Duration = Duration::from_secs(30);

/// Topic / service / action bridge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    RosToBus,
    BusToRos,
    Both,
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

/// Whitelisted ROS action kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActKind {
    Fibonacci,
}

impl ActKind {
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Fibonacci => "example_interfaces/action/Fibonacci",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "example_interfaces/action/Fibonacci" => Ok(Self::Fibonacci),
            other => Err(BusError::Protocol(format!(
                "unsupported ros2 bridge action type {other:?}; supported: example_interfaces/action/Fibonacci"
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

#[cfg(test)]
mod act_kind_tests {
    use super::*;

    #[test]
    fn parse_fibonacci() {
        assert_eq!(
            ActKind::parse("example_interfaces/action/Fibonacci").unwrap(),
            ActKind::Fibonacci
        );
        assert!(ActKind::parse("example_interfaces/action/Other").is_err());
    }

    #[test]
    fn reject_both_for_actions() {
        let res = Ros2Bridge::new("t").add_action(
            "/fib",
            "/fib",
            ActKind::Fibonacci,
            Direction::Both,
        );
        assert!(res.is_err());
        let msg = res.err().unwrap().to_string();
        assert!(msg.contains("both"));
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RouteSpec {
    ros_topic: String,
    bus_topic: String,
    type_name: String,
    direction: Direction,
}

#[derive(Debug, Clone)]
pub(crate) struct ServiceRouteSpec {
    ros_service: String,
    bus_service: String,
    kind: SrvKind,
    direction: Direction,
}

#[derive(Debug, Clone)]
pub(crate) struct ActionRouteSpec {
    ros_action: String,
    bus_action: String,
    kind: ActKind,
    direction: Direction,
}

/// Fluent builder: `Ros2Bridge::new(...).bus_tcp(...).route(...).type_name(...).add().build()?`.
pub struct Ros2BridgeBuilder {
    name: String,
    bus_options: NodeOptions,
    routes: Vec<RouteSpec>,
    services: Vec<ServiceRouteSpec>,
    actions: Vec<ActionRouteSpec>,
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
    /// Keeps typed ROS services / clients / action entities alive for the bridge lifetime.
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
    type_name: Option<String>,
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

/// Intermediate action route configuration before [`ActionRouteBuilder::add`].
pub struct ActionRouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_action: String,
    bus_action: String,
    kind: Option<ActKind>,
    direction: Direction,
}

impl Ros2Bridge {
    pub fn new(name: impl Into<String>) -> Ros2BridgeBuilder {
        Ros2BridgeBuilder {
            name: name.into(),
            bus_options: NodeOptions::tcp(),
            routes: Vec::new(),
            services: Vec::new(),
            actions: Vec::new(),
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
        // ROS executor runs on a background thread so Bus→ROS service/action handlers
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
            type_name: None,
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

    pub fn action(
        self,
        ros_action: impl Into<String>,
        bus_action: impl Into<String>,
    ) -> ActionRouteBuilder {
        ActionRouteBuilder {
            parent: self,
            ros_action: ros_action.into(),
            bus_action: bus_action.into(),
            kind: None,
            direction: Direction::RosToBus,
        }
    }

    pub(crate) fn push_route(
        mut self,
        ros_topic: String,
        bus_topic: String,
        type_name: String,
        direction: Direction,
    ) -> Result<Self> {
        codec::lookup_topic_codec(&type_name)?;
        self.routes.push(RouteSpec {
            ros_topic,
            bus_topic,
            type_name,
            direction,
        });
        Ok(self)
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

    pub(crate) fn push_action(
        mut self,
        ros_action: String,
        bus_action: String,
        kind: ActKind,
        direction: Direction,
    ) -> Result<Self> {
        if matches!(direction, Direction::Both) {
            return Err(BusError::Protocol(
                "ros2 bridge actions do not support direction=both; use ros_to_bus or bus_to_ros"
                    .into(),
            ));
        }
        self.actions.push(ActionRouteSpec {
            ros_action,
            bus_action,
            kind,
            direction,
        });
        Ok(self)
    }

    /// Add a topic route by ROS type string (e.g. `sensor_msgs/msg/Image`).
    pub fn add_route(
        self,
        ros_topic: impl Into<String>,
        bus_topic: impl Into<String>,
        type_name: impl Into<String>,
        direction: Direction,
    ) -> Result<Self> {
        self.push_route(
            ros_topic.into(),
            bus_topic.into(),
            type_name.into(),
            direction,
        )
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

    /// Add a typed action route (for FFI / non-fluent callers).
    pub fn add_action(
        self,
        ros_action: impl Into<String>,
        bus_action: impl Into<String>,
        kind: ActKind,
        direction: Direction,
    ) -> Result<Self> {
        self.push_action(
            ros_action.into(),
            bus_action.into(),
            kind,
            direction,
        )
    }

    pub fn build(self) -> Result<Ros2Bridge> {
        if self.routes.is_empty() && self.services.is_empty() && self.actions.is_empty() {
            return Err(BusError::Protocol(
                "Ros2Bridge requires at least one topic route, service, or action".into(),
            ));
        }

        let context = RosContext::default_from_env().map_err(|e| {
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

        for act in &self.actions {
            wire_action_route(&ros_node, &mut bus_node, act, &mut ros_entities)?;
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
    pub fn string(self) -> Self {
        self.type_name("std_msgs/msg/String")
    }

    pub fn imu(self) -> Self {
        self.type_name("sensor_msgs/msg/Imu")
    }

    pub fn image(self) -> Self {
        self.type_name("sensor_msgs/msg/Image")
    }

    pub fn compressed_video(self) -> Self {
        self.type_name("foxglove_msgs/msg/CompressedVideo")
    }

    /// Set an arbitrary registered ROS type (e.g. `sensor_msgs/msg/Image`).
    pub fn type_name(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        let type_name = self.type_name.ok_or_else(|| {
            BusError::Protocol(
                "ros2 bridge route: call .type_name(...) or .string()/.imu()/.image() before .add()"
                    .into(),
            )
        })?;
        self.parent
            .push_route(self.ros_topic, self.bus_topic, type_name, self.direction)
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

impl ActionRouteBuilder {
    pub fn fibonacci(mut self) -> Self {
        self.kind = Some(ActKind::Fibonacci);
        self
    }

    pub fn act_kind(mut self, kind: ActKind) -> Self {
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
                "ros2 bridge action: call .fibonacci() before .add()".into(),
            )
        })?;
        self.parent
            .push_action(self.ros_action, self.bus_action, kind, self.direction)
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
    let codec: &'static dyn TopicCodec = codec::lookup_topic_codec(&route.type_name)?;
    let type_name = codec.ros_type();
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
        let tx = ros_to_bus_tx.clone();
        let bus_topic_cb = bus_topic.clone();
        let sub = ros_node
            .create_dynamic_subscription(
                type_name,
                ros_topic.as_str(),
                move |dyn_msg, _info| {
                    let payload = match codec.ros_to_bus(&dyn_msg) {
                        Ok(p) => p,
                        Err(e) => {
                            log::warn!("ros→bus {} convert: {e}", codec.type_name());
                            return;
                        }
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
        let cb: MessageCallback = Arc::new(move |_topic, payload| {
            if let Ok(mut g) = echo.lock() {
                if g.is_echo(payload) {
                    return;
                }
                g.remember(payload);
            }
            match codec.bus_to_ros(payload) {
                Ok(dyn_msg) => {
                    if let Err(e) = ros_pub.publish(dyn_msg) {
                        log::warn!("bus→ros {} publish: {e}", codec.type_name());
                    }
                }
                Err(e) => log::warn!("bus→ros {} convert: {e}", codec.type_name()),
            }
        });
        bus_node.create_subscription_raw(bus_topic.as_str(), cb, None)?;
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

fn wire_action_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    route: &ActionRouteSpec,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    match (route.kind, route.direction) {
        (ActKind::Fibonacci, Direction::RosToBus) => {
            let bus_client = Arc::new(Mutex::new(
                bus_node.create_action_client::<BusFibonacci>(route.bus_action.as_str())?,
            ));
            let timeout = ACTION_CALL_TIMEOUT;
            let srv = ros_node
                .create_action_server::<ros_act::Fibonacci, _>(
                    route.ros_action.as_str(),
                    move |requested| {
                        let bus_client = Arc::clone(&bus_client);
                        async move {
                            let goal = (**requested.goal()).clone();
                            let accepted = requested.accept();
                            let executing = match accepted.begin() {
                                BeginAcceptedGoal::Execute(e) => e,
                                BeginAcceptedGoal::Cancel(c) => {
                                    return c.cancelled_with(ros_act::Fibonacci_Result {
                                        sequence: Vec::new(),
                                    });
                                }
                            };
                            let bus_goal = convert::fibonacci_ros_goal_to_bus(&goal);
                            let call = tokio::task::spawn_blocking(move || {
                                let guard = bus_client.lock().map_err(|e| {
                                    format!("bus action client lock poisoned: {e}")
                                })?;
                                guard
                                    .send_goal(&bus_goal, None, Some(timeout))
                                    .map_err(|e| e.to_string())
                            })
                            .await;
                            match call {
                                Ok(Ok(outcome)) => {
                                    for fb in &outcome.feedbacks {
                                        executing.publish_feedback(
                                            convert::fibonacci_bus_feedback_to_ros(fb),
                                        );
                                    }
                                    executing.succeeded_with(convert::fibonacci_bus_result_to_ros(
                                        &outcome.result,
                                    ))
                                }
                                Ok(Err(e)) => {
                                    log::warn!("ros→bus Fibonacci goal failed: {e}");
                                    executing.aborted_with(ros_act::Fibonacci_Result {
                                        sequence: Vec::new(),
                                    })
                                }
                                Err(e) => {
                                    log::warn!("ros→bus Fibonacci join failed: {e}");
                                    executing.aborted_with(ros_act::Fibonacci_Result {
                                        sequence: Vec::new(),
                                    })
                                }
                            }
                        }
                    },
                )
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_server Fibonacci: {e}"))
                })?;
            ros_entities.push(Box::new(srv));
        }
        (ActKind::Fibonacci, Direction::BusToRos) => {
            let ros_client = ros_node
                .create_action_client::<ros_act::Fibonacci>(route.ros_action.as_str())
                .map_err(|e| {
                    BusError::Protocol(format!("ros create_action_client Fibonacci: {e}"))
                })?;
            ros_entities.push(Box::new(Arc::clone(&ros_client)));
            let timeout = ACTION_CALL_TIMEOUT;
            let handler: ActionGoalHandler = Arc::new(move |body| {
                let bus_goal = match BusFibonacciGoal::decode(body) {
                    Ok(g) => g,
                    Err(e) => {
                        log::warn!("decode FibonacciGoal: {e}");
                        return vec![(
                            "RESULT".into(),
                            BusFibonacciResult {
                                sequence: Vec::new(),
                            }
                            .encode_to_vec(),
                        )];
                    }
                };
                match call_ros_fibonacci(&ros_client, &bus_goal, timeout) {
                    Ok(replies) => replies,
                    Err(msg) => {
                        log::warn!("bus→ros Fibonacci failed: {msg}");
                        vec![(
                            "RESULT".into(),
                            BusFibonacciResult {
                                sequence: Vec::new(),
                            }
                            .encode_to_vec(),
                        )]
                    }
                }
            });
            let _ = bus_node.create_action_server_raw(route.bus_action.as_str(), handler, None)?;
        }
        (_, Direction::Both) => {
            return Err(BusError::Protocol(
                "ros2 bridge actions do not support direction=both".into(),
            ));
        }
    }
    Ok(())
}

fn noop_raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        noop_raw_waker()
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
    )
}

fn poll_once<F: Future + Unpin>(fut: &mut F) -> Poll<F::Output> {
    let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    Pin::new(fut).poll(&mut cx)
}

fn await_with_timeout<F: Future + Unpin>(
    mut fut: F,
    timeout: Duration,
) -> std::result::Result<F::Output, String> {
    let deadline = Instant::now() + timeout;
    loop {
        match poll_once(&mut fut) {
            Poll::Ready(v) => return Ok(v),
            Poll::Pending => {
                if Instant::now() >= deadline {
                    return Err("timed out waiting for ROS action".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn call_ros_fibonacci(
    client: &rclrs::ActionClient<ros_act::Fibonacci>,
    bus_goal: &BusFibonacciGoal,
    timeout: Duration,
) -> std::result::Result<Vec<(String, Vec<u8>)>, String> {
    let ros_goal = convert::fibonacci_bus_goal_to_ros(bus_goal);
    let requested = client
        .try_request_goal(ros_goal)
        .map_err(|e| format!("ros Fibonacci request_goal: {e}"))?;
    let goal_client = match await_with_timeout(requested, timeout)? {
        Some(gc) => gc,
        None => return Err("ROS action server rejected Fibonacci goal".into()),
    };
    let GoalClient {
        mut feedback,
        result,
        ..
    } = goal_client;
    let mut replies = Vec::new();
    let deadline = Instant::now() + timeout;
    let mut result_fut = result;
    loop {
        while let Ok(fb) = feedback.try_recv() {
            let bus_fb = convert::fibonacci_ros_feedback_to_bus(&fb);
            replies.push(("FEEDBACK".into(), bus_fb.encode_to_vec()));
        }
        match poll_once(&mut result_fut) {
            Poll::Ready((_status, res)) => {
                let bus_res = convert::fibonacci_ros_result_to_bus(&res);
                replies.push(("RESULT".into(), bus_res.encode_to_vec()));
                return Ok(replies);
            }
            Poll::Pending => {
                if Instant::now() >= deadline {
                    return Err("timed out waiting for ROS Fibonacci result".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
