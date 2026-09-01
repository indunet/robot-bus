use std::any::Any;
use std::time::Duration;

use crate::errors::BusError;
use crate::ros2_bridge::mapper::{Direction, TopicMapper, TopicWireContext};
use crate::runtime::TopicPublisherRaw;

use super::*;

fn ros_qos() -> TopicQos {
    TopicQos::keep_last(10).reliable()
}

fn bus_qos() -> TopicQos {
    TopicQos::keep_last(8).best_effort()
}

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
        _qos: TopicQos,
    ) -> std::result::Result<Box<dyn Any + Send + Sync>, BusError> {
        Err(BusError::Protocol("dummy".into()))
    }
    fn attach_bus_to_ros(&self, _ctx: TopicWireContext<'_>) -> std::result::Result<(), BusError> {
        Err(BusError::Protocol("dummy".into()))
    }
}

#[test]
fn per_route_custom_topic_mapper_accepted() {
    Ros2Bridge::new("t")
        .from_ros("/a", ros_qos())
        .to_bus("/a", bus_qos())
        .mapper(DummyTopicMapper)
        .add()
        .expect("custom mapper should add");
}

#[test]
fn builtin_concrete_mapper() {
    Ros2Bridge::new("t")
        .from_ros("/a", ros_qos())
        .to_bus("/a", bus_qos())
        .mapper(crate::ros2_bridge::StdMsgsStringMapper)
        .add()
        .expect("builtin topic mapper object");
}

#[test]
fn builtin_service_concrete_mapper() {
    Ros2Bridge::new("t")
        .service()
        .from_ros("/a", ros_qos())
        .to_bus("/a", bus_qos())
        .mapper(crate::ros2_bridge::TriggerServiceMapper)
        .add()
        .expect("builtin service mapper object");
}

#[test]
fn builtin_service_timeout_override() {
    Ros2Bridge::new("t")
        .service()
        .from_ros("/a", ros_qos())
        .to_bus("/a", bus_qos())
        .mapper(crate::ros2_bridge::TriggerServiceMapper)
        .timeout(Duration::from_millis(250))
        .add()
        .expect("timeout should be accepted at add()");
}

#[test]
fn service_from_bus_to_ros() {
    let b = Ros2Bridge::new("t")
        .service()
        .from_bus("/a", bus_qos())
        .to_ros("/a", ros_qos())
        .mapper(crate::ros2_bridge::TriggerServiceMapper)
        .add()
        .expect("add");
    assert_eq!(b.services[0].direction, Direction::BusToRos2);
    assert_eq!(b.services[0].ros_qos, ros_qos());
    assert_eq!(b.services[0].bus_qos, bus_qos());
}

#[test]
fn lazy_defaults_off() {
    let b = Ros2Bridge::new("t")
        .from_ros("/a", ros_qos())
        .to_bus("/a", bus_qos())
        .mapper(DummyTopicMapper)
        .add()
        .expect("add");
    assert!(!b.routes[0].lazy);
}

#[test]
fn lazy_opt_in_ros2_to_bus() {
    let b = Ros2Bridge::new("t")
        .from_ros("/cam", ros_qos())
        .to_bus("/cam", bus_qos())
        .mapper(DummyTopicMapper)
        .lazy()
        .add()
        .expect("lazy add");
    assert!(b.routes[0].lazy);
    assert_eq!(b.routes[0].direction, Direction::Ros2ToBus);
}

#[test]
fn from_bus_to_ros() {
    let b = Ros2Bridge::new("t")
        .from_bus("/a", bus_qos())
        .to_ros("/a", ros_qos())
        .mapper(DummyTopicMapper)
        .add()
        .expect("add");
    assert_eq!(b.routes[0].direction, Direction::BusToRos2);
    assert!(!b.routes[0].lazy);
}

#[test]
fn lazy_and_eager_routes_independent() {
    let b = Ros2Bridge::new("t")
        .from_ros("/a", ros_qos())
        .to_bus("/a", bus_qos())
        .mapper(DummyTopicMapper)
        .add()
        .unwrap()
        .from_ros("/b", ros_qos())
        .to_bus("/b", bus_qos())
        .mapper(DummyTopicMapper)
        .lazy()
        .add()
        .unwrap();
    assert!(!b.routes[0].lazy);
    assert!(b.routes[1].lazy);
}

#[test]
fn qos_stored_per_endpoint() {
    let ros = TopicQos::keep_last(20).best_effort();
    let bus = TopicQos::keep_last(4).best_effort();
    let b = Ros2Bridge::new("t")
        .from_ros("/a", ros)
        .to_bus("/a", bus)
        .mapper(DummyTopicMapper)
        .add()
        .expect("add");
    assert_eq!(b.routes[0].ros_qos, ros);
    assert_eq!(b.routes[0].bus_qos, bus);

    let latched = TopicQos::keep_last(1).reliable().transient_local();
    let b2 = Ros2Bridge::new("t")
        .from_ros("/tf_static", latched)
        .to_bus("/tf_static", bus)
        .mapper(DummyTopicMapper)
        .add()
        .expect("add");
    assert_eq!(b2.routes[0].ros_qos, latched);
    assert!(b2.routes[0].ros_qos.is_transient_local());
}

#[test]
fn bus_reliable_rejected() {
    let err = Ros2Bridge::new("t")
        .from_ros("/a", ros_qos())
        .to_bus("/a", TopicQos::keep_last(8).reliable())
        .mapper(DummyTopicMapper)
        .add()
        .err()
        .expect("should fail")
        .to_string();
    assert!(err.contains("best_effort"), "{err}");
}

#[test]
fn service_bus_reliable_rejected() {
    let err = Ros2Bridge::new("t")
        .service()
        .from_ros("/a", ros_qos())
        .to_bus("/a", TopicQos::keep_last(8).reliable())
        .mapper(crate::ros2_bridge::TriggerServiceMapper)
        .add()
        .err()
        .expect("should fail")
        .to_string();
    assert!(err.contains("best_effort"), "{err}");
}
