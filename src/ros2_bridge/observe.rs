//! Startup route dump, 1 Hz [`BridgeSnapshot`], and idle ConsoleEvent.

use std::sync::Arc;
use std::time::Duration;

use prost::Message;

use crate::robot_bus_interfaces::msg::v1::{BridgeRoute, BridgeSnapshot, ConsoleEvent};
use crate::ros2_bridge::drop_stats::{RouteHealth, unix_ms};
use crate::ros2_bridge::mapper::{Direction, TopicQos};

pub const IDLE_GRACE: Duration = Duration::from_secs(15);
pub const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(1);

/// Static route row plus live [`RouteHealth`] for console / idle.
pub struct ConsoleRoute {
    pub kind: &'static str,
    pub direction: Direction,
    pub ros_name: String,
    pub bus_name: String,
    pub type_name: String,
    pub ros_qos: String,
    pub bus_qos: String,
    pub lazy: bool,
    pub watch_idle: bool,
    pub health: Arc<RouteHealth>,
}

impl ConsoleRoute {
    pub fn topic(
        ros_name: impl Into<String>,
        bus_name: impl Into<String>,
        direction: Direction,
        type_name: impl Into<String>,
        ros_qos: TopicQos,
        bus_qos: TopicQos,
        lazy: bool,
        health: Arc<RouteHealth>,
    ) -> Self {
        Self {
            kind: "topic",
            direction,
            ros_name: ros_name.into(),
            bus_name: bus_name.into(),
            type_name: type_name.into(),
            ros_qos: ros_qos.console_label(),
            bus_qos: bus_qos.console_label(),
            lazy,
            watch_idle: true,
            health,
        }
    }

    pub fn rpc(
        kind: &'static str,
        ros_name: impl Into<String>,
        bus_name: impl Into<String>,
        direction: Direction,
        type_name: impl Into<String>,
        ros_qos: TopicQos,
        bus_qos: TopicQos,
    ) -> Self {
        Self {
            kind,
            direction,
            ros_name: ros_name.into(),
            bus_name: bus_name.into(),
            type_name: type_name.into(),
            ros_qos: ros_qos.console_label(),
            bus_qos: bus_qos.console_label(),
            lazy: false,
            watch_idle: false,
            health: Arc::new(RouteHealth::new()),
        }
    }

    pub fn log_line(&self) -> String {
        let ty = if self.type_name.is_empty() {
            "-"
        } else {
            self.type_name.as_str()
        };
        let lazy = if self.lazy { "  lazy" } else { "" };
        format!(
            "{:<7} {:<8} {} → {}  {}  ros={}  bus={}{lazy}",
            self.kind,
            self.direction.console_label(),
            self.ros_name,
            self.bus_name,
            ty,
            self.ros_qos,
            self.bus_qos,
        )
    }

    pub fn to_proto(&self, enabled: bool, grace_elapsed: bool) -> BridgeRoute {
        let idle = self.watch_idle && self.health.is_idle(enabled, grace_elapsed);
        BridgeRoute {
            kind: self.kind.to_string(),
            direction: self.direction.console_label().to_string(),
            ros_name: self.ros_name.clone(),
            bus_name: self.bus_name.clone(),
            type_name: self.type_name.clone(),
            ros_qos: self.ros_qos.clone(),
            bus_qos: self.bus_qos.clone(),
            lazy: self.lazy,
            enabled,
            rx: self.health.rx(),
            tx: self.health.tx(),
            convert_fail: self.health.convert_fail(),
            decode_fail: self.health.decode_fail(),
            publish_fail: self.health.publish_fail(),
            last_rx_ms: self.health.last_rx_ms(),
            idle,
        }
    }

    pub fn idle_message(&self) -> String {
        format!(
            "no traffic on {} {} for {}s; possible wrong direction or ROS QoS mismatch",
            self.direction.console_label(),
            self.ros_name,
            IDLE_GRACE.as_secs(),
        )
    }
}

pub fn log_route_table(bridge_name: &str, routes: &[ConsoleRoute]) {
    let mut lines = Vec::with_capacity(routes.len() + 1);
    lines.push(format!("ros2_bridge '{bridge_name}' routes:"));
    for route in routes {
        lines.push(format!("  {}", route.log_line()));
    }
    log::info!("{}", lines.join("\n"));
}

pub fn encode_snapshot(
    bridge_id: &str,
    bridge_name: &str,
    routes: &[ConsoleRoute],
    enabled: impl Fn(&ConsoleRoute) -> bool,
    grace_elapsed: bool,
) -> Vec<u8> {
    BridgeSnapshot {
        bridge_id: bridge_id.to_string(),
        bridge_name: bridge_name.to_string(),
        routes: routes
            .iter()
            .map(|r| r.to_proto(enabled(r), grace_elapsed))
            .collect(),
    }
    .encode_to_vec()
}

pub fn encode_idle_event(bridge_name: &str, event_id: u64, route: &ConsoleRoute) -> Vec<u8> {
    ConsoleEvent {
        id: format!("bridge-idle-{event_id}"),
        ts: unix_ms(),
        level: "WARN".into(),
        source: format!("ros2_bridge/{bridge_name}"),
        message: route.idle_message(),
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ros2_bridge::mapper::{Direction, TopicQos};
    use std::sync::Arc;

    #[test]
    fn topic_row_log_line_and_idle_proto() {
        let health = Arc::new(RouteHealth::new());
        let row = ConsoleRoute::topic(
            "/cam",
            "/cam",
            Direction::Ros2ToBus,
            "SensorMsgsImageMapper",
            TopicQos::keep_last(10).best_effort(),
            TopicQos::keep_last(8).best_effort(),
            true,
            health,
        );
        let line = row.log_line();
        assert!(line.contains("topic"));
        assert!(line.contains("ros→bus"));
        assert!(line.contains("/cam"));
        assert!(line.contains("lazy"));
        let proto = row.to_proto(true, true);
        assert!(proto.idle);
        assert!(proto.lazy);
        assert_eq!(proto.kind, "topic");
    }
}
