//! Fixed system topic names for broker console introspection.

/// Namespace for console snapshot topics and control-plane services.
pub const PREFIX: &str = "/robot_bus";

/// Periodic [`crate::robot_bus_interfaces::msg::v1::BrokerStatus`] snapshot.
pub const STATUS: &str = "/robot_bus/status";
/// Periodic [`crate::robot_bus_interfaces::msg::v1::TopicStatsList`] snapshot.
pub const TOPICS: &str = "/robot_bus/topics";
/// Periodic [`crate::robot_bus_interfaces::msg::v1::ServiceStatsList`] snapshot.
pub const SERVICES: &str = "/robot_bus/services";
/// Periodic [`crate::robot_bus_interfaces::msg::v1::ActionStatsList`] snapshot.
pub const ACTIONS: &str = "/robot_bus/actions";
/// Periodic [`crate::robot_bus_interfaces::msg::v1::TopologySnapshot`] snapshot.
pub const TOPOLOGY: &str = "/robot_bus/topology";
/// Streaming [`crate::robot_bus_interfaces::msg::v1::ConsoleEvent`] log lines.
pub const EVENTS: &str = "/robot_bus/events";
/// Immediate [`crate::robot_bus_interfaces::msg::v1::TopicDemand`] (subscriber count).
pub const TOPIC_DEMAND: &str = "/robot_bus/topic_demand";

/// Client → broker: [`crate::robot_bus_interfaces::msg::v1::TopologyRegister`].
pub const TOPOLOGY_REGISTER: &str = "/robot_bus/topology/register";
/// Client → broker: [`crate::robot_bus_interfaces::msg::v1::TopologyUnregister`].
pub const TOPOLOGY_UNREGISTER: &str = "/robot_bus/topology/unregister";
/// Client → broker: [`crate::robot_bus_interfaces::msg::v1::TopicTypeRegister`].
pub const TOPIC_TYPE_REGISTER: &str = "/robot_bus/topic_type/register";

/// All control-plane topics the broker control subscriber must join.
pub const CONTROL_SUBSCRIBE: &[&str] =
    &[TOPOLOGY_REGISTER, TOPOLOGY_UNREGISTER, TOPIC_TYPE_REGISTER];

/// Snapshot topics the broker status publisher emits.
pub const SNAPSHOT_PUBLISH: &[&str] = &[STATUS, TOPICS, SERVICES, ACTIONS, TOPOLOGY, EVENTS];

/// Built-in TANK endpoints (`/robot_bus/tank/*`).
pub const TANK_PREFIX: &str = "/robot_bus/tank";

/// True for names under the console namespace (`/robot_bus` and `/robot_bus/*`).
///
/// Used to keep per-broker snapshots local: message federation does not forward
/// this prefix. Nodes may still pub/sub these names.
pub fn is_reserved_name(name: &str) -> bool {
    let name = name.trim();
    name == PREFIX || name.starts_with("/robot_bus/")
}

/// True for built-in tank demo names.
pub fn is_builtin_tank_name(name: &str) -> bool {
    let name = name.trim();
    name == TANK_PREFIX || name.starts_with("/robot_bus/tank/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_namespace() {
        assert!(is_reserved_name("/robot_bus"));
        assert!(is_reserved_name("/robot_bus/status"));
        assert!(is_reserved_name(" /robot_bus/topology/register "));
        assert!(is_reserved_name("/robot_bus/tank/pose"));
        assert!(!is_reserved_name("/robot_bus_extra"));
        assert!(!is_reserved_name("/robot1/imu"));
        assert!(is_builtin_tank_name("/robot_bus/tank/pose"));
        assert!(!is_builtin_tank_name("/robot_bus/status"));
        assert!(!is_builtin_tank_name("/robot_bus/bot_extra"));
    }
}
