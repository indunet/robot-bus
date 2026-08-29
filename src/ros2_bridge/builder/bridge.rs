use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::errors::{BusError, Result};
use crate::lazy_subscribe::{should_enable_ros_subscription, CONSOLE_DETECT_TIMEOUT};
use crate::runtime::{Node, TopicPublisherRaw};

use super::specs::DemandEvent;
use super::wire::create_ros2_to_bus_sub;

/// In-process dual-stack bridge (ROS 2 + robot-bus).
pub struct Ros2Bridge {
    pub(super) bus_node: Node,
    pub(super) ros_node: rclrs::Node,
    pub(super) bus_pubs: HashMap<String, TopicPublisherRaw>,
    pub(super) lazy_routes: HashMap<String, super::specs::LazyRos2ToBus>,
    pub(super) demand_rx: Receiver<DemandEvent>,
    pub(super) subscriber_counts: HashMap<String, u32>,
    /// `None` until a console snapshot arrives or [`CONSOLE_DETECT_TIMEOUT`] elapses.
    pub(super) console_live: Option<bool>,
    pub(super) first_spin_at: Option<Instant>,
    pub(super) _demand_subs: Vec<crate::runtime::SubscriptionHandle>,
    pub(super) eager_bus_topics: HashSet<String>,
    pub(super) _ros_subs: Vec<Box<dyn Any + Send + Sync>>,
    /// Keeps typed ROS services / clients / action entities alive for the bridge lifetime.
    pub(super) _ros_entities: Vec<Box<dyn Any + Send + Sync>>,
    pub(super) ros_commands: Arc<rclrs::ExecutorCommands>,
    pub(super) _ros_spin: Option<JoinHandle<()>>,
}

impl Drop for Ros2Bridge {
    fn drop(&mut self) {
        self.ros_commands.halt_spinning();
        if let Some(h) = self._ros_spin.take() {
            let _ = h.join();
        }
    }
}

impl Ros2Bridge {
    /// Whether this bridge currently holds a ROS subscription for `bus_topic`.
    ///
    /// Eager ROS2→bus routes are `true` immediately after [`super::Ros2BridgeBuilder::build`].
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
                route.ros_qos,
            ) {
                Ok(sub) => route.sub = Some(sub),
                Err(err) => log::warn!("lazy ros2 subscribe {topic}: {err}"),
            }
        }
    }
}
