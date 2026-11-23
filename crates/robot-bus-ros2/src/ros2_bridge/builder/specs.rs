use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::errors::{BusError, Result};
use crate::ros2_bridge::mapper::{ActionMapper, Direction, ServiceMapper, TopicMapper, TopicQos};

/// Default timeout for bridged service calls (ROS↔bus).
pub const SERVICE_CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for bridged action goals (ROS↔bus).
pub const ACTION_CALL_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct RouteSpec {
    pub(crate) ros_topic: String,
    pub(crate) bus_topic: String,
    pub(crate) mapper: Arc<dyn TopicMapper>,
    pub(crate) type_name: String,
    pub(crate) direction: Direction,
    pub(crate) lazy: bool,
    pub(crate) ros_qos: TopicQos,
    pub(crate) bus_qos: TopicQos,
}

pub(crate) struct LazyRos2ToBus {
    pub(crate) ros_topic: String,
    pub(crate) mapper: Arc<dyn TopicMapper>,
    pub(crate) ros_qos: TopicQos,
    pub(crate) sub: Option<Box<dyn Any + Send + Sync>>,
    pub(crate) health: Arc<crate::ros2_bridge::drop_stats::RouteHealth>,
}

pub(crate) enum DemandEvent {
    Count { topic: String, subscribers: u32 },
    Snapshot { counts: HashMap<String, u32> },
}

pub(crate) struct ServiceRouteSpec {
    pub(crate) ros_service: String,
    pub(crate) bus_service: String,
    pub(crate) mapper: Arc<dyn ServiceMapper>,
    pub(crate) direction: Direction,
    pub(crate) timeout: Duration,
    pub(crate) ros_qos: TopicQos,
    pub(crate) bus_qos: TopicQos,
}

pub(crate) struct ActionRouteSpec {
    pub(crate) ros_action: String,
    pub(crate) bus_action: String,
    pub(crate) mapper: Arc<dyn ActionMapper>,
    pub(crate) direction: Direction,
    pub(crate) timeout: Duration,
    pub(crate) ros_qos: TopicQos,
    pub(crate) bus_qos: TopicQos,
}

pub(crate) fn reject_bus_reliable(qos: TopicQos) -> Result<()> {
    if qos.is_reliable() {
        Err(BusError::Protocol(
            "ros2 bridge: bus TopicQos must be .best_effort() \
             (bus has no DDS reliability)"
                .into(),
        ))
    } else {
        Ok(())
    }
}
