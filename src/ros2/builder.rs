//! Chained builder for [`Ros2Bridge`].

use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use prost::Message as ProstMessage;
use rclrs::{
    Context, CreateBasicExecutor, DynamicPublisher, DynamicSubscription, Executor, MessageTypeName,
    SpinOptions,
};

use crate::discovery::DiscoverOpts;
use crate::errors::{BusError, Result};
use crate::runtime::{Node, NodeOptions, TopicPublisherRaw};
use crate::sensor_msgs::msg::v1::Imu as BusImu;
use crate::std_msgs::msg::v1::String as BusString;

use super::convert;
use super::echo::EchoFilter;
use super::yaml;

/// Topic bridge direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    RosToBus,
    BusToRos,
    Both,
}

/// Whitelisted ROS message kinds.
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

#[derive(Debug, Clone)]
pub(crate) struct RouteSpec {
    ros_topic: String,
    bus_topic: String,
    kind: MsgKind,
    direction: Direction,
}

/// Fluent builder: `Ros2Bridge::new(...).bus_tcp(...).route(...).string().add().build()?`.
pub struct Ros2BridgeBuilder {
    name: String,
    bus_options: NodeOptions,
    routes: Vec<RouteSpec>,
}

/// In-process dual-stack bridge (ROS 2 + robot-bus).
pub struct Ros2Bridge {
    ros_executor: Executor,
    bus_node: Node,
    /// ROS→bus payloads (ZMQ publisher is not `Send`; drain on the spin thread).
    ros_to_bus_rx: Receiver<(String, Vec<u8>)>,
    bus_pubs: std::collections::HashMap<String, TopicPublisherRaw>,
    _echo: Arc<Mutex<EchoFilter>>,
    _ros_subs: Vec<DynamicSubscription>,
    _ros_pubs: Vec<DynamicPublisher>,
}

/// Intermediate route configuration before [`RouteBuilder::add`].
pub struct RouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_topic: String,
    bus_topic: String,
    kind: Option<MsgKind>,
    direction: Direction,
}

impl Ros2Bridge {
    pub fn new(name: impl Into<String>) -> Ros2BridgeBuilder {
        Ros2BridgeBuilder {
            name: name.into(),
            bus_options: NodeOptions::tcp(),
            routes: Vec::new(),
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
        let _ = self
            .ros_executor
            .spin(SpinOptions::spin_once().timeout(timeout));
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

    /// Add a typed route (for FFI / non-fluent callers).
    pub fn add_route(
        self,
        ros_topic: impl Into<String>,
        bus_topic: impl Into<String>,
        kind: MsgKind,
        direction: Direction,
    ) -> Self {
        self.push_route(ros_topic.into(), bus_topic.into(), kind, direction)
    }

    pub fn build(self) -> Result<Ros2Bridge> {
        if self.routes.is_empty() {
            return Err(BusError::Protocol(
                "Ros2Bridge requires at least one route".into(),
            ));
        }

        let context = Context::default_from_env().map_err(|e| {
            BusError::Protocol(format!(
                "rclrs Context::default_from_env failed ({e}); source ROS 2 first"
            ))
        })?;
        let ros_executor = context.create_basic_executor();
        let ros_node = ros_executor
            .create_node(self.name.as_str())
            .map_err(|e| BusError::Protocol(format!("rclrs create_node: {e}")))?;

        let mut bus_node = Node::with_options(format!("{}_bus", self.name), self.bus_options);
        let echo = Arc::new(Mutex::new(EchoFilter::new(Duration::from_millis(500))));
        let (ros_to_bus_tx, ros_to_bus_rx) = mpsc::sync_channel::<(String, Vec<u8>)>(1024);

        let mut ros_subs = Vec::new();
        let mut ros_pubs = Vec::new();
        let mut bus_pubs = std::collections::HashMap::new();

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

        Ok(Ros2Bridge {
            ros_executor,
            bus_node,
            ros_to_bus_rx,
            bus_pubs,
            _echo: echo,
            _ros_subs: ros_subs,
            _ros_pubs: ros_pubs,
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
