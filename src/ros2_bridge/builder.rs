//! Chained builder for [`Ros2Bridge`].
//!
//! Configuration is code-only: attach concrete mapper objects via `.mapper(...)`.
//! There is no YAML loader and no type-name-string route API.

use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use prost::Message;
use rclrs::{Context as RosContext, CreateBasicExecutor, SpinOptions};

use crate::console_topics;
use crate::discovery::DiscoverOpts;
use crate::errors::{BusError, Result};
use crate::lazy_subscribe::{should_enable_ros_subscription, CONSOLE_DETECT_TIMEOUT};
use crate::robot_bus_interfaces::msg::v1::{TopicDemand, TopicStatsList};
use crate::runtime::{
    MessageCallback, Node, NodeOptions, QosProfile, SubscriptionHandle, TopicPublisherRaw,
};

use super::mapper::{
    ActionMapper, ActionWireContext, Direction, ServiceMapper, ServiceWireContext, TopicMapper,
    TopicWireContext,
};

/// Default timeout for bridged service calls (ROS↔bus).
pub const SERVICE_CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for bridged action goals (ROS↔bus).
pub const ACTION_CALL_TIMEOUT: Duration = Duration::from_secs(30);

pub use super::mapper::TopicRouteQos;

pub(crate) struct RouteSpec {
    ros_topic: String,
    bus_topic: String,
    mapper: Arc<dyn TopicMapper>,
    direction: Direction,
    lazy: bool,
    qos: TopicRouteQos,
}

struct LazyRos2ToBus {
    ros_topic: String,
    mapper: Arc<dyn TopicMapper>,
    qos: TopicRouteQos,
    sub: Option<Box<dyn Any + Send + Sync>>,
}

enum DemandEvent {
    Count { topic: String, subscribers: u32 },
    Snapshot { counts: HashMap<String, u32> },
}

pub(crate) struct ServiceRouteSpec {
    ros_service: String,
    bus_service: String,
    mapper: Arc<dyn ServiceMapper>,
    direction: Direction,
    timeout: Duration,
}

pub(crate) struct ActionRouteSpec {
    ros_action: String,
    bus_action: String,
    mapper: Arc<dyn ActionMapper>,
    direction: Direction,
    timeout: Duration,
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
    ros_node: rclrs::Node,
    bus_pubs: HashMap<String, TopicPublisherRaw>,
    lazy_routes: HashMap<String, LazyRos2ToBus>,
    demand_rx: Receiver<DemandEvent>,
    subscriber_counts: HashMap<String, u32>,
    /// `None` until a console snapshot arrives or [`CONSOLE_DETECT_TIMEOUT`] elapses.
    console_live: Option<bool>,
    first_spin_at: Option<Instant>,
    _demand_subs: Vec<SubscriptionHandle>,
    eager_bus_topics: HashSet<String>,
    _ros_subs: Vec<Box<dyn Any + Send + Sync>>,
    /// Keeps typed ROS services / clients / action entities alive for the bridge lifetime.
    _ros_entities: Vec<Box<dyn Any + Send + Sync>>,
    ros_commands: Arc<rclrs::ExecutorCommands>,
    _ros_spin: Option<JoinHandle<()>>,
}

impl Drop for Ros2Bridge {
    fn drop(&mut self) {
        self.ros_commands.halt_spinning();
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
    lazy: bool,
    qos: TopicRouteQos,
}

/// Intermediate service route configuration before [`ServiceRouteBuilder::add`].
pub struct ServiceRouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_service: String,
    bus_service: String,
    mapper: Option<Arc<dyn ServiceMapper>>,
    direction: Direction,
    timeout: Duration,
}

/// Intermediate action route configuration before [`ActionRouteBuilder::add`].
pub struct ActionRouteBuilder {
    parent: Ros2BridgeBuilder,
    ros_action: String,
    bus_action: String,
    mapper: Option<Arc<dyn ActionMapper>>,
    direction: Direction,
    timeout: Duration,
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

    /// Whether this bridge currently holds a ROS subscription for `bus_topic`.
    ///
    /// Eager ROS2→bus routes are `true` immediately after [`Ros2BridgeBuilder::build`].
    /// Lazy routes follow bus subscriber demand (or the no-console fallback).
    pub fn has_ros_subscription(&self, bus_topic: &str) -> bool {
        if let Some(route) = self.lazy_routes.get(bus_topic) {
            route.sub.is_some()
        } else {
            self.eager_bus_topics.contains(bus_topic)
        }
    }

    pub fn spin(&mut self) -> Result<()> {
        loop {
            self.spin_once_inner(None)?;
        }
    }

    pub fn spin_once(&mut self, timeout: Duration) -> Result<()> {
        self.spin_once_inner(Some(timeout))
    }

    fn spin_once_inner(&mut self, timeout: Option<Duration>) -> Result<()> {
        // ROS executor runs on a background thread so Bus→ROS service/action handlers
        // can wait on client Promises without nested-spin deadlocks.
        if self.first_spin_at.is_none() {
            self.first_spin_at = Some(Instant::now());
        }
        let spin_result = match self.bus_node.spin_once(timeout) {
            Err(BusError::Protocol(msg)) if msg.contains("nothing registered") => Ok(()),
            other => other.map(|_| ()),
        };
        self.drain_demand();
        if self.console_live.is_none() {
            if let Some(started) = self.first_spin_at {
                if started.elapsed() >= CONSOLE_DETECT_TIMEOUT {
                    self.console_live = Some(false);
                    self.apply_lazy_routes();
                }
            }
        }
        spin_result
    }

    fn drain_demand(&mut self) {
        let mut dirty = false;
        while let Ok(event) = self.demand_rx.try_recv() {
            self.console_live = Some(true);
            dirty = true;
            match event {
                DemandEvent::Count { topic, subscribers } => {
                    self.subscriber_counts.insert(topic, subscribers);
                }
                DemandEvent::Snapshot { counts } => {
                    for (topic, n) in counts {
                        self.subscriber_counts.insert(topic, n);
                    }
                }
            }
        }
        if dirty {
            self.apply_lazy_routes();
        }
    }

    fn apply_lazy_routes(&mut self) {
        let console_live = self.console_live;
        let counts = &self.subscriber_counts;
        let ros_node = self.ros_node.clone();
        let mut to_enable: Vec<String> = Vec::new();
        let mut to_disable: Vec<String> = Vec::new();
        for (bus_topic, route) in &self.lazy_routes {
            let n = counts.get(bus_topic).copied().unwrap_or(0);
            let want = should_enable_ros_subscription(true, console_live, n);
            match (want, route.sub.is_some()) {
                (true, false) => to_enable.push(bus_topic.clone()),
                (false, true) => to_disable.push(bus_topic.clone()),
                _ => {}
            }
        }
        for topic in to_disable {
            if let Some(route) = self.lazy_routes.get_mut(&topic) {
                route.sub = None;
            }
        }
        for topic in to_enable {
            let Some(bus_pub) = self.bus_pubs.get(&topic).cloned() else {
                continue;
            };
            let Some(route) = self.lazy_routes.get_mut(&topic) else {
                continue;
            };
            match create_ros2_to_bus_sub(
                &ros_node,
                bus_pub,
                &route.mapper,
                &route.ros_topic,
                route.qos,
            ) {
                Ok(sub) => route.sub = Some(sub),
                Err(err) => log::warn!("lazy ros2 subscribe {topic}: {err}"),
            }
        }
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

    /// Override bus [`NodeOptions`] (tests / custom binds). Prefer `bus_tcp` /
    /// `bus_discover` for normal use.
    pub fn bus_options(mut self, options: NodeOptions) -> Self {
        self.bus_options = options;
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
            lazy: false,
            qos: TopicRouteQos::default(),
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
            timeout: SERVICE_CALL_TIMEOUT,
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
            timeout: ACTION_CALL_TIMEOUT,
        }
    }

    pub(crate) fn push_route(
        mut self,
        ros_topic: String,
        bus_topic: String,
        mapper: Arc<dyn TopicMapper>,
        direction: Direction,
        lazy: bool,
        qos: TopicRouteQos,
    ) -> Result<Self> {
        if lazy && direction != Direction::Ros2ToBus {
            return Err(BusError::Protocol(
                "ros2 bridge route: .lazy() is only valid for Direction::Ros2ToBus".into(),
            ));
        }
        self.routes.push(RouteSpec {
            ros_topic,
            bus_topic,
            mapper,
            direction,
            lazy,
            qos,
        });
        Ok(self)
    }

    pub(crate) fn push_service(
        mut self,
        ros_service: String,
        bus_service: String,
        mapper: Arc<dyn ServiceMapper>,
        direction: Direction,
        timeout: Duration,
    ) -> Result<Self> {
        self.services.push(ServiceRouteSpec {
            ros_service,
            bus_service,
            mapper,
            direction,
            timeout,
        });
        Ok(self)
    }

    pub(crate) fn push_action(
        mut self,
        ros_action: String,
        bus_action: String,
        mapper: Arc<dyn ActionMapper>,
        direction: Direction,
        timeout: Duration,
    ) -> Result<Self> {
        self.actions.push(ActionRouteSpec {
            ros_action,
            bus_action,
            mapper,
            direction,
            timeout,
        });
        Ok(self)
    }

    /// Add a topic route with an explicit [`TopicMapper`].
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
            false,
            TopicRouteQos::default(),
        )
    }

    /// Add a service route with an explicit [`ServiceMapper`].
    pub fn add_service_mapper(
        self,
        ros_service: impl Into<String>,
        bus_service: impl Into<String>,
        mapper: impl IntoServiceMapper,
        direction: Direction,
        timeout: Duration,
    ) -> Result<Self> {
        self.push_service(
            ros_service.into(),
            bus_service.into(),
            mapper.into_service_mapper(),
            direction,
            timeout,
        )
    }

    /// Add an action route with an explicit [`ActionMapper`].
    pub fn add_action_mapper(
        self,
        ros_action: impl Into<String>,
        bus_action: impl Into<String>,
        mapper: impl IntoActionMapper,
        direction: Direction,
        timeout: Duration,
    ) -> Result<Self> {
        self.push_action(
            ros_action.into(),
            bus_action.into(),
            mapper.into_action_mapper(),
            direction,
            timeout,
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

        let mut ros_subs = Vec::new();
        let mut bus_pubs = HashMap::new();
        let mut lazy_routes: HashMap<String, LazyRos2ToBus> = HashMap::new();
        let mut eager_bus_topics = HashSet::new();
        let mut ros_entities: Vec<Box<dyn Any + Send + Sync>> = Vec::new();

        for route in &self.routes {
            wire_route(
                &ros_node,
                &mut bus_node,
                &mut bus_pubs,
                &mut lazy_routes,
                &mut eager_bus_topics,
                route,
                &mut ros_subs,
                &mut ros_entities,
            )?;
        }

        for svc in &self.services {
            wire_service_route(&ros_node, &mut bus_node, svc, &mut ros_entities)?;
        }

        for act in &self.actions {
            wire_action_route(&ros_node, &mut bus_node, act, &mut ros_entities)?;
        }

        let (demand_tx, demand_rx) = mpsc::channel();
        let mut demand_subs = Vec::new();
        if !lazy_routes.is_empty() {
            demand_subs.extend(subscribe_demand(&mut bus_node, demand_tx)?);
        }

        let ros_commands = Arc::clone(ros_executor.commands());
        let ros_spin = thread::Builder::new()
            .name("ros2_bridge_spin".into())
            .spawn(move || {
                let _ = ros_executor.spin(SpinOptions::default());
            })
            .map_err(|e| BusError::Protocol(format!("spawn ros2 spin thread: {e}")))?;

        Ok(Ros2Bridge {
            bus_node,
            ros_node,
            bus_pubs,
            lazy_routes,
            demand_rx,
            subscriber_counts: HashMap::new(),
            console_live: None,
            first_spin_at: None,
            _demand_subs: demand_subs,
            eager_bus_topics,
            _ros_subs: ros_subs,
            _ros_entities: ros_entities,
            ros_commands,
            _ros_spin: Some(ros_spin),
        })
    }
}

/// Accept either a concrete [`TopicMapper`] or an [`Arc<dyn TopicMapper>`].
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

    /// Opt-in lazy ROS 2 subscription for **ROS2→bus** topics only.
    ///
    /// Default is eager: `build()` creates the ROS subscription immediately so
    /// the ROS graph shows the bridge. `.lazy()` waits until at least one
    /// robot-bus subscriber exists for this bus topic.
    pub fn lazy(mut self) -> Self {
        self.lazy = true;
        self
    }

    /// ROS KeepLast(`n`) plus bus topic HWM `n`. Does not change reliability.
    pub fn qos_depth(mut self, n: i32) -> Self {
        self.qos.depth = Some(n);
        self
    }

    /// ROS reliability best-effort (sensor-style). Bus reliability is unchanged.
    pub fn best_effort(mut self) -> Self {
        self.qos.best_effort = true;
        self
    }

    /// Best-effort KeepLast(5) on ROS, bus depth 5. Opt-in; does not change route defaults.
    pub fn sensor_data(mut self) -> Self {
        self.qos = TopicRouteQos::SENSOR_DATA;
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
        self.parent.push_route(
            self.ros_topic,
            self.bus_topic,
            mapper,
            self.direction,
            self.lazy,
            self.qos,
        )
    }
}

impl ServiceRouteBuilder {
    /// Attach a service codec for this route (e.g. [`TriggerServiceMapper`] or custom).
    pub fn mapper(mut self, mapper: impl IntoServiceMapper) -> Self {
        self.mapper = Some(mapper.into_service_mapper());
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Override the default service call timeout ([`SERVICE_CALL_TIMEOUT`]).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
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
        self.parent.push_service(
            self.ros_service,
            self.bus_service,
            mapper,
            self.direction,
            self.timeout,
        )
    }
}

impl ActionRouteBuilder {
    /// Attach an action codec for this route (e.g. [`FibonacciActionMapper`] or custom).
    pub fn mapper(mut self, mapper: impl IntoActionMapper) -> Self {
        self.mapper = Some(mapper.into_action_mapper());
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Override the default action goal timeout ([`ACTION_CALL_TIMEOUT`]).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
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
        self.parent.push_action(
            self.ros_action,
            self.bus_action,
            mapper,
            self.direction,
            self.timeout,
        )
    }
}

fn create_ros2_to_bus_publisher(
    bus_node: &mut Node,
    bus_topic: &str,
    qos: TopicRouteQos,
) -> Result<TopicPublisherRaw> {
    let pub_ = if let Some(depth) = qos.depth {
        bus_node.create_publisher_raw_with_qos(bus_topic, QosProfile::keep_last(depth))?
    } else {
        bus_node.create_publisher_raw(bus_topic)?
    };
    if let Err(e) = pub_.set_send_timeout_ms(0) {
        log::warn!("ros→bus {bus_topic} send timeout: {e}");
    }
    Ok(pub_)
}

fn create_ros2_to_bus_sub(
    ros_node: &rclrs::Node,
    bus_pub: TopicPublisherRaw,
    mapper: &Arc<dyn TopicMapper>,
    ros_topic: &str,
    qos: TopicRouteQos,
) -> Result<Box<dyn Any + Send + Sync>> {
    mapper.create_ros2_to_bus_subscription(ros_node, bus_pub, ros_topic, qos)
}

fn subscribe_demand(
    bus_node: &mut Node,
    demand_tx: Sender<DemandEvent>,
) -> Result<Vec<SubscriptionHandle>> {
    let tx_demand = demand_tx.clone();
    let demand_cb: MessageCallback =
        Arc::new(move |_topic, payload| match TopicDemand::decode(payload) {
            Ok(msg) => {
                let _ = tx_demand.send(DemandEvent::Count {
                    topic: msg.topic,
                    subscribers: msg.subscribers,
                });
            }
            Err(err) => log::warn!("decode TopicDemand: {err}"),
        });
    let h1 = bus_node.create_subscription_raw(console_topics::TOPIC_DEMAND, demand_cb, None)?;

    let tx_topics = demand_tx;
    let topics_cb: MessageCallback = Arc::new(move |_topic, payload| match TopicStatsList::decode(
        payload,
    ) {
        Ok(list) => {
            let counts = list
                .topics
                .into_iter()
                .map(|t| (t.name, t.subscribers as u32))
                .collect();
            let _ = tx_topics.send(DemandEvent::Snapshot { counts });
        }
        Err(err) => log::warn!("decode TopicStatsList: {err}"),
    });
    let h2 = bus_node.create_subscription_raw(console_topics::TOPICS, topics_cb, None)?;
    Ok(vec![h1, h2])
}

fn wire_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    bus_pubs: &mut HashMap<String, TopicPublisherRaw>,
    lazy_routes: &mut HashMap<String, LazyRos2ToBus>,
    eager_bus_topics: &mut HashSet<String>,
    route: &RouteSpec,
    ros_subs: &mut Vec<Box<dyn Any + Send + Sync>>,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    let mapper = Arc::clone(&route.mapper);
    let ros_topic = route.ros_topic.clone();
    let bus_topic = route.bus_topic.clone();
    let qos = route.qos;

    match route.direction {
        Direction::BusToRos2 => {
            mapper.attach_bus_to_ros(TopicWireContext {
                ros_node,
                bus_node,
                ros_topic: ros_topic.as_str(),
                bus_topic: bus_topic.as_str(),
                qos,
                ros_entities,
            })?;
        }
        Direction::Ros2ToBus => {
            let bus_pub = create_ros2_to_bus_publisher(bus_node, bus_topic.as_str(), qos)?;
            bus_pubs.insert(bus_topic.clone(), bus_pub.clone());
            if route.lazy {
                lazy_routes.insert(
                    bus_topic,
                    LazyRos2ToBus {
                        ros_topic,
                        mapper,
                        qos,
                        sub: None,
                    },
                );
            } else {
                let sub = create_ros2_to_bus_sub(ros_node, bus_pub, &mapper, &ros_topic, qos)?;
                ros_subs.push(sub);
                eager_bus_topics.insert(bus_topic);
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
    route.mapper.attach(ServiceWireContext {
        ros_node,
        bus_node,
        ros_service: route.ros_service.as_str(),
        bus_service: route.bus_service.as_str(),
        direction: route.direction,
        timeout: route.timeout,
        ros_entities,
    })
}

fn wire_action_route(
    ros_node: &rclrs::Node,
    bus_node: &mut Node,
    route: &ActionRouteSpec,
    ros_entities: &mut Vec<Box<dyn Any + Send + Sync>>,
) -> Result<()> {
    route.mapper.attach(ActionWireContext {
        ros_node,
        bus_node,
        ros_action: route.ros_action.as_str(),
        bus_action: route.bus_action.as_str(),
        direction: route.direction,
        timeout: route.timeout,
        ros_entities,
    })
}

#[cfg(test)]
mod route_mapper_tests {
    use super::*;

    struct DummyTopicMapper;
    impl TopicMapper for DummyTopicMapper {
        fn type_name(&self) -> &'static str {
            "test_msgs/msg/Dummy"
        }
        fn create_ros2_to_bus_subscription(
            &self,
            _ros_node: &rclrs::Node,
            _bus_pub: TopicPublisherRaw,
            _ros_topic: &str,
            _qos: TopicRouteQos,
        ) -> std::result::Result<Box<dyn Any + Send + Sync>, BusError> {
            Err(BusError::Protocol("dummy".into()))
        }
        fn attach_bus_to_ros(
            &self,
            _ctx: TopicWireContext<'_>,
        ) -> std::result::Result<(), BusError> {
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
            .mapper(crate::ros2_bridge::TriggerServiceMapper)
            .add()
            .expect("builtin service mapper object");
    }

    #[test]
    fn builtin_service_timeout_override() {
        Ros2Bridge::new("t")
            .service("/a", "/a")
            .mapper(crate::ros2_bridge::TriggerServiceMapper)
            .timeout(Duration::from_millis(250))
            .add()
            .expect("timeout should be accepted at add()");
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

    #[test]
    fn lazy_defaults_off() {
        let b = Ros2Bridge::new("t")
            .route("/a", "/a")
            .mapper(DummyTopicMapper)
            .add()
            .expect("add");
        assert!(!b.routes[0].lazy);
    }

    #[test]
    fn lazy_opt_in_ros2_to_bus() {
        let b = Ros2Bridge::new("t")
            .route("/cam", "/cam")
            .mapper(DummyTopicMapper)
            .lazy()
            .add()
            .expect("lazy add");
        assert!(b.routes[0].lazy);
        assert_eq!(b.routes[0].direction, Direction::Ros2ToBus);
    }

    #[test]
    fn lazy_rejects_bus_to_ros2() {
        let err = Ros2Bridge::new("t")
            .route("/a", "/a")
            .mapper(DummyTopicMapper)
            .direction(Direction::BusToRos2)
            .lazy()
            .add()
            .err()
            .expect("should fail")
            .to_string();
        assert!(err.contains("lazy"), "{err}");
        assert!(err.contains("Ros2ToBus"), "{err}");
    }

    #[test]
    fn lazy_and_eager_routes_independent() {
        let b = Ros2Bridge::new("t")
            .route("/a", "/a")
            .mapper(DummyTopicMapper)
            .add()
            .unwrap()
            .route("/b", "/b")
            .mapper(DummyTopicMapper)
            .lazy()
            .add()
            .unwrap();
        assert!(!b.routes[0].lazy);
        assert!(b.routes[1].lazy);
    }

    #[test]
    fn qos_defaults_off() {
        let b = Ros2Bridge::new("t")
            .route("/a", "/a")
            .mapper(DummyTopicMapper)
            .add()
            .expect("add");
        assert_eq!(b.routes[0].qos, TopicRouteQos::default());
    }

    #[test]
    fn qos_depth_and_best_effort() {
        let b = Ros2Bridge::new("t")
            .route("/a", "/a")
            .mapper(DummyTopicMapper)
            .qos_depth(20)
            .best_effort()
            .add()
            .expect("add");
        assert_eq!(b.routes[0].qos.depth, Some(20));
        assert!(b.routes[0].qos.best_effort);
        assert!(!b.routes[0].qos.sensor_data);
    }

    #[test]
    fn qos_sensor_data() {
        let b = Ros2Bridge::new("t")
            .route("/cam", "/cam")
            .mapper(DummyTopicMapper)
            .sensor_data()
            .add()
            .expect("add");
        assert_eq!(b.routes[0].qos, TopicRouteQos::SENSOR_DATA);
    }
}
