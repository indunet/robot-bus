//! Fixed system topic names for broker console introspection.

/// Periodic [`crate::robot_bus_interface::msg::v1::BrokerStatus`] snapshot.
pub const STATUS: &str = "/_robot_bus/status";
/// Periodic [`crate::robot_bus_interface::msg::v1::TopicStatsList`] snapshot.
pub const TOPICS: &str = "/_robot_bus/topics";
/// Periodic [`crate::robot_bus_interface::msg::v1::ServiceStatsList`] snapshot.
pub const SERVICES: &str = "/_robot_bus/services";
/// Periodic [`crate::robot_bus_interface::msg::v1::ActionStatsList`] snapshot.
pub const ACTIONS: &str = "/_robot_bus/actions";
/// Periodic [`crate::robot_bus_interface::msg::v1::TopologySnapshot`] snapshot.
pub const TOPOLOGY: &str = "/_robot_bus/topology";
/// Streaming [`crate::robot_bus_interface::msg::v1::ConsoleEvent`] log lines.
pub const EVENTS: &str = "/_robot_bus/events";

/// Client → broker: [`crate::robot_bus_interface::msg::v1::TopologyRegister`].
pub const TOPOLOGY_REGISTER: &str = "/_robot_bus/topology/register";
/// Client → broker: [`crate::robot_bus_interface::msg::v1::TopologyUnregister`].
pub const TOPOLOGY_UNREGISTER: &str = "/_robot_bus/topology/unregister";
/// Client → broker: [`crate::robot_bus_interface::msg::v1::TopicTypeRegister`].
pub const TOPIC_TYPE_REGISTER: &str = "/_robot_bus/topic_type/register";

/// All control-plane topics the broker control subscriber must join.
pub const CONTROL_SUBSCRIBE: &[&str] =
    &[TOPOLOGY_REGISTER, TOPOLOGY_UNREGISTER, TOPIC_TYPE_REGISTER];

/// Snapshot topics the broker status publisher emits.
pub const SNAPSHOT_PUBLISH: &[&str] = &[STATUS, TOPICS, SERVICES, ACTIONS, TOPOLOGY, EVENTS];
