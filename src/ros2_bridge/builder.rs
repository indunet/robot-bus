//! Chained builder for [`Ros2Bridge`].

use std::any::Any;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rclrs::{
    Context as RosContext, CreateBasicExecutor, DynamicPublisher, DynamicSubscription, SpinOptions,
};

use crate::discovery::DiscoverOpts;
use crate::errors::{BusError, Result};
use crate::runtime::{MessageCallback, Node, NodeOptions, TopicPublisherRaw};

use super::mapper::{
    self, ActionMapper, ActionWireContext, Direction, ServiceMapper, ServiceWireContext,
    TopicMapper,
};
use super::mappers::action_bridges;
use super::mappers::service_bridges;
use super::yaml;

/// Default timeout for bridged service calls (ROS↔bus).
pub const SERVICE_CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for bridged action goals (ROS↔bus).
pub const ACTION_CALL_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct RouteSpec {
    ros_topic: String,
    bus_topic: String,
    mapper: Arc<dyn TopicMapper>,
    direction: Direction,
}

pub(crate) struct ServiceRouteSpec {
    ros_service: String,
    bus_service: String,
    mapper: Arc<dyn ServiceMapper>,
    direction: Direction,
}

pub(crate) struct ActionRouteSpec {
    ros_action: String,
    bus_action: String,
    mapper: Arc<dyn ActionMapper>,
    direction: Direction,
}

/// Fluent builder: `Ros2Bridge::new(...).bus_tcp(...).route(...).mapper(...).add().build()?`.
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
    mapper: Option<Arc<dyn TopicMapper>>,
    direction: Direction,
}

/// Intermediate service route configuration before [`ServiceRouteBuilder::add`].
pub struct ServiceRouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_service: String,
    bus_service: String,
    mapper: Option<Arc<dyn ServiceMapper>>,
    direction: Direction,
}

/// Intermediate action route configuration before [`ActionRouteBuilder::add`].
pub struct ActionRouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_action: String,
    bus_action: String,
    mapper: Option<Arc<dyn ActionMapper>>,
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

    /// Discover a broker via HTTP API, then connect over TCP.
    pub fn bus_discover(self, api_url: impl Into<String>) -> Result<Self> {
        self.bus_discover_ex(api_url, None, None)
    }

    /// Discover with optional timeout (seconds) and broker id filter.
    pub fn bus_discover_ex(
        mut self,
        api_url: impl Into<String>,
        timeout_secs: Option<f64>,
        broker_id: Option<String>,
    ) -> Result<Self> {
        let mut opts = DiscoverOpts {
            api_url: api_url.into(),
            ..Default::default()
        };
        if let Some(t) = timeout_secs {
            opts.timeout = Duration::from_secs_f64(t);
        }
        opts.broker_id = broker_id;
        self.bus_options = NodeOptions::tcp().discover(opts)?;
        Ok(self)
    }

    pub fn route(self, ros_topic: impl Into<String>, bus_topic: impl Into<String>) -> RouteBuilder {
        RouteBuilder {
            parent: self,
            ros_topic: ros_topic.into(),
            bus_topic: bus_topic.into(),
            mapper: None,
            direction: Direction::Ros2ToBus,
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
            mapper: None,
            direction: Direction::Ros2ToBus,
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
            mapper: None,
            direction: Direction::Ros2ToBus,
        }
    }

    pub(crate) fn push_route(
        mut self,
        ros_topic: String,
        bus_topic: String,
        mapper: Arc<dyn TopicMapper>,
        direction: Direction,
    ) -> Result<Self> {
        self.routes.push(RouteSpec {
            ros_topic,
            bus_topic,
            mapper,
            direction,
        });
        Ok(self)
    }

    pub(crate) fn push_service(
        mut self,
        ros_service: String,
        bus_service: String,
        mapper: Arc<dyn ServiceMapper>,
        direction: Direction,
    ) -> Result<Self> {
        self.services.push(ServiceRouteSpec {
            ros_service,
            bus_service,
            mapper,
            direction,
        });
        Ok(self)
    }

    pub(crate) fn push_action(
        mut self,
        ros_action: String,
        bus_action: String,
        mapper: Arc<dyn ActionMapper>,
        direction: Direction,
    ) -> Result<Self> {
        self.actions.push(ActionRouteSpec {
            ros_action,
            bus_action,
            mapper,
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
        let type_name = type_name.into();
        let mapper = mapper::lookup_topic_mapper_arc(&type_name)?;
        self.push_route(ros_topic.into(), bus_topic.into(), mapper, direction)
    }

    /// Add a topic route with an explicit [`TopicMapper`] (builtin or custom / FFI).
    pub fn add_route_mapper(
        self,
        ros_topic: impl Into<String>,
        bus_topic: impl Into<String>,
        mapper: impl IntoTopicMapper,
        direction: Direction,
    ) -> Result<Self> {
        self.push_route(
            ros_topic.into(),
            bus_topic.into(),
            mapper.into_topic_mapper(),
            direction,
        )
    }

    /// Add a service route by ROS type string (e.g. `std_srvs/srv/Trigger`).
    pub fn add_service(
        self,
        ros_service: impl Into<String>,
        bus_service: impl Into<String>,
        type_name: impl Into<String>,
        direction: Direction,
    ) -> Result<Self> {
        let mapper = service_bridges::lookup_service_mapper(&type_name.into())?;
        self.push_service(ros_service.into(), bus_service.into(), mapper, direction)
    }

    /// Add an action route by ROS type string (e.g. `example_interfaces/action/Fibonacci`).
    pub fn add_action(
        self,
        ros_action: impl Into<String>,
        bus_action: impl Into<String>,
        type_name: impl Into<String>,
        direction: Direction,
    ) -> Result<Self> {
        let mapper = action_bridges::lookup_action_mapper(&type_name.into())?;
        self.push_action(ros_action.into(), bus_action.into(), mapper, direction)
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
        let (ros_to_bus_tx, ros_to_bus_rx) = mpsc::sync_channel::<(String, Vec<u8>)>(1024);

        let mut ros_subs = Vec::new();
        let mut ros_pubs = Vec::new();
        let mut bus_pubs = std::collections::HashMap::new();
        let mut ros_entities: Vec<Box<dyn Any + Send + Sync>> = Vec::new();

        for route in &self.routes {
            wire_route(
                &ros_node,
                &mut bus_node,
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
                    let _ = ros_executor
                        .spin(SpinOptions::spin_once().timeout(Duration::from_millis(10)));
                }
            })
            .map_err(|e| BusError::Protocol(format!("spawn ros2 spin thread: {e}")))?;

        Ok(Ros2Bridge {
            bus_node,
            ros_to_bus_rx,
            bus_pubs,
            _ros_subs: ros_subs,
            _ros_pubs: ros_pubs,
            _ros_entities: ros_entities,
            ros_halt,
            _ros_spin: Some(ros_spin),
        })
    }
}



/// Accept either a concrete [`TopicMapper`] or an [`Arc<dyn TopicMapper>`] (e.g. from
/// [`mapper::lookup_topic_mapper_arc`]).
pub trait IntoTopicMapper {
    fn into_topic_mapper(self) -> Arc<dyn TopicMapper>;
}

impl<T: TopicMapper + 'static> IntoTopicMapper for T {
    fn into_topic_mapper(self) -> Arc<dyn TopicMapper> {
        Arc::new(self)
    }
}

impl IntoTopicMapper for Arc<dyn TopicMapper> {
    fn into_topic_mapper(self) -> Arc<dyn TopicMapper> {
        self
    }
}

pub trait IntoServiceMapper {
    fn into_service_mapper(self) -> Arc<dyn ServiceMapper>;
}

impl<T: ServiceMapper + 'static> IntoServiceMapper for T {
    fn into_service_mapper(self) -> Arc<dyn ServiceMapper> {
        Arc::new(self)
    }
}

impl IntoServiceMapper for Arc<dyn ServiceMapper> {
    fn into_service_mapper(self) -> Arc<dyn ServiceMapper> {
        self
    }
}

pub trait IntoActionMapper {
    fn into_action_mapper(self) -> Arc<dyn ActionMapper>;
}

impl<T: ActionMapper + 'static> IntoActionMapper for T {
    fn into_action_mapper(self) -> Arc<dyn ActionMapper> {
        Arc::new(self)
    }
}

impl IntoActionMapper for Arc<dyn ActionMapper> {
    fn into_action_mapper(self) -> Arc<dyn ActionMapper> {
        self
    }
}

impl RouteBuilder {
    /// Attach a topic mapper for this route (builtin ZST or custom [`TopicMapper`]).
    pub fn mapper(mut self, mapper: impl IntoTopicMapper) -> Self {
        self.mapper = Some(mapper.into_topic_mapper());
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        let mapper = self.mapper.ok_or_else(|| {
            BusError::Protocol(
                "ros2 bridge route: call .mapper(...) before .add() \
                 (e.g. StdMsgsStringMapper or your TopicMapper)"
                    .into(),
            )
        })?;
        self.parent
            .push_route(self.ros_topic, self.bus_topic, mapper, self.direction)
    }
}

impl ServiceRouteBuilder {
    /// Attach a service mapper for this route (e.g. [`TriggerServiceMapper`] or custom).
    pub fn mapper(mut self, mapper: impl IntoServiceMapper) -> Self {
        self.mapper = Some(mapper.into_service_mapper());
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        let mapper = self.mapper.ok_or_else(|| {
            BusError::Protocol(
                "ros2 bridge service: call .mapper(...) before .add() \
                 (e.g. TriggerServiceMapper or your ServiceMapper)"
                    .into(),
            )
        })?;
        self.parent
            .push_service(self.ros_service, self.bus_service, mapper, self.direction)
    }
}

impl ActionRouteBuilder {
    /// Attach an action mapper for this route (e.g. [`FibonacciActionMapper`] or custom).
    pub fn mapper(mut self, mapper: impl IntoActionMapper) -> Self {
        self.mapper = Some(mapper.into_action_mapper());
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        let mapper = self.mapper.ok_or_else(|| {
            BusError::Protocol(
                "ros2 bridge action: call .mapper(...) before .add() \
                 (e.g. FibonacciActionMapper or your ActionMapper)"
                    .into(),
            )
        })?;
        self.parent
            .push_action(self.ros_action, self.bus_action, mapper, self.direction)
    }
}

fn wire_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    ros_to_bus_tx: &SyncSender<(String, Vec<u8>)>,
    bus_pubs: &mut std::collections::HashMap<String, TopicPublisherRaw>,
    route: &RouteSpec,
    ros_subs: &mut Vec<DynamicSubscription>,
    ros_pubs: &mut Vec<DynamicPublisher>,
) -> Result<()> {
    let mapper = Arc::clone(&route.mapper);
    let type_name = mapper.ros_type();
    let ros_topic = route.ros_topic.clone();
    let bus_topic = route.bus_topic.clone();

    match route.direction {
        Direction::BusToRos2 => {
            let ros_pub = ros_node
                .create_dynamic_publisher(type_name, ros_topic.as_str())
                .map_err(|e| BusError::Protocol(format!("ros dynamic publisher: {e}")))?;
            ros_pubs.push(ros_pub.clone());
            let mapper = Arc::clone(&mapper);
            let cb: MessageCallback = Arc::new(move |_topic, payload| {
                match mapper.bus_to_ros(payload) {
                    Ok(dyn_msg) => {
                        if let Err(e) = ros_pub.publish(dyn_msg) {
                            log::warn!("bus→ros {} publish: {e}", mapper.type_name());
                        }
                    }
                    Err(e) => log::warn!("bus→ros {} convert: {e}", mapper.type_name()),
                }
            });
            bus_node.create_subscription_raw(bus_topic.as_str(), cb, None)?;
        }
        Direction::Ros2ToBus => {
            bus_pubs.insert(
                bus_topic.clone(),
                bus_node.create_publisher_raw(bus_topic.as_str())?,
            );
            let tx = ros_to_bus_tx.clone();
            let bus_topic_cb = bus_topic.clone();
            let mapper = Arc::clone(&mapper);
            let sub = ros_node
                .create_dynamic_subscription(type_name, ros_topic.as_str(), move |dyn_msg, _info| {
                    let payload = match mapper.ros_to_bus(&dyn_msg) {
                        Ok(p) => p,
                        Err(e) => {
                            log::warn!("ros→bus {} convert: {e}", mapper.type_name());
                            return;
                        }
                    };
                    if let Err(e) = tx.send((bus_topic_cb.clone(), payload)) {
                        log::warn!("ros→bus channel: {e}");
                    }
                })
                .map_err(|e| BusError::Protocol(format!("ros dynamic subscription: {e}")))?;
            ros_subs.push(sub);
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
    route.mapper.wire(ServiceWireContext {
        ros_node,
        bus_node,
        ros_service: route.ros_service.as_str(),
        bus_service: route.bus_service.as_str(),
        direction: route.direction,
        timeout: SERVICE_CALL_TIMEOUT,
        ros_entities,
    })
}

fn wire_action_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    route: &ActionRouteSpec,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    route.mapper.wire(ActionWireContext {
        ros_node,
        bus_node,
        ros_action: route.ros_action.as_str(),
        bus_action: route.bus_action.as_str(),
        direction: route.direction,
        timeout: ACTION_CALL_TIMEOUT,
        ros_entities,
    })
}


#[cfg(test)]
mod route_mapper_tests {
    use super::*;
    use rclrs::DynamicMessage;

    struct DummyTopicMapper;
    impl TopicMapper for DummyTopicMapper {
        fn type_name(&self) -> &'static str {
            "test_msgs/msg/Dummy"
        }
        fn ros_to_bus(&self, _msg: &DynamicMessage) -> std::result::Result<Vec<u8>, BusError> {
            Ok(Vec::new())
        }
        fn bus_to_ros(&self, _payload: &[u8]) -> std::result::Result<DynamicMessage, BusError> {
            Err(BusError::Protocol("dummy".into()))
        }
    }

    #[test]
    fn per_route_custom_topic_mapper_accepted() {
        Ros2Bridge::new("t")
            .route("/a", "/a")
            .mapper(DummyTopicMapper)
            .direction(Direction::Ros2ToBus)
            .add()
            .expect("custom mapper should add");
    }

    #[test]
    fn builtin_concrete_mapper() {
        Ros2Bridge::new("t")
            .route("/a", "/a")
            .mapper(crate::ros2_bridge::StdMsgsStringMapper)
            .add()
            .expect("builtin topic mapper object");
    }

    #[test]
    fn builtin_service_concrete_mapper() {
        Ros2Bridge::new("t")
            .service("/a", "/a")
            .mapper(service_bridges::TriggerServiceMapper)
            .add()
            .expect("builtin service mapper object");
    }

    #[test]
    fn missing_mapper_fails() {
        let err = Ros2Bridge::new("t")
            .route("/a", "/a")
            .add()
            .err()
            .expect("should fail")
            .to_string();
        assert!(err.contains("mapper"), "{err}");
    }
}
