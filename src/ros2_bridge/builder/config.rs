use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rclrs::{Context as RosContext, CreateBasicExecutor, SpinOptions};

use crate::discovery::DiscoverOpts;
use crate::errors::{BusError, Result};
use crate::ros2_bridge::drop_stats::DropStats;
use crate::ros2_bridge::mapper::{ActionMapper, Direction, ServiceMapper, TopicMapper};
use crate::runtime::{Node, NodeOptions};

use super::bridge::Ros2Bridge;
use super::specs::{
    reject_bus_reliable, ActionRouteSpec, LazyRos2ToBus, RouteSpec, ServiceRouteSpec,
};
use super::wire::{subscribe_demand, wire_action_route, wire_route, wire_service_route};
use super::{IntoTopicMapper, TopicQos};

/// Fluent builder: `Ros2Bridge::new(...).from_ros(...).to_bus(...).mapper(...).add()`.
pub struct Ros2BridgeBuilder {
    name: String,
    bus_options: NodeOptions,
    pub(crate) routes: Vec<RouteSpec>,
    pub(crate) services: Vec<ServiceRouteSpec>,
    pub(crate) actions: Vec<ActionRouteSpec>,
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

    pub(crate) fn push_route(
        mut self,
        ros_topic: String,
        bus_topic: String,
        mapper: Arc<dyn TopicMapper>,
        direction: Direction,
        lazy: bool,
        ros_qos: TopicQos,
        bus_qos: TopicQos,
    ) -> Result<Self> {
        reject_bus_reliable(bus_qos)?;
        if lazy && direction != Direction::Ros2ToBus {
            return Err(BusError::Protocol(
                "ros2 bridge route: .lazy() is only valid for ROS2→bus".into(),
            ));
        }
        self.routes.push(RouteSpec {
            ros_topic,
            bus_topic,
            mapper,
            direction,
            lazy,
            ros_qos,
            bus_qos,
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
        ros_qos: TopicQos,
        bus_qos: TopicQos,
    ) -> Result<Self> {
        reject_bus_reliable(bus_qos)?;
        self.services.push(ServiceRouteSpec {
            ros_service,
            bus_service,
            mapper,
            direction,
            timeout,
            ros_qos,
            bus_qos,
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
        ros_qos: TopicQos,
        bus_qos: TopicQos,
    ) -> Result<Self> {
        reject_bus_reliable(bus_qos)?;
        self.actions.push(ActionRouteSpec {
            ros_action,
            bus_action,
            mapper,
            direction,
            timeout,
            ros_qos,
            bus_qos,
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
        ros_qos: TopicQos,
        bus_qos: TopicQos,
    ) -> Result<Self> {
        self.push_route(
            ros_topic.into(),
            bus_topic.into(),
            mapper.into_topic_mapper(),
            direction,
            false,
            ros_qos,
            bus_qos,
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
        let drop_stats = Arc::new(DropStats::new());

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
                Arc::clone(&drop_stats),
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
            drop_stats,
        })
    }
}
