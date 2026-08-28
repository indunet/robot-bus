use std::sync::Arc;

use crate::errors::Result;
use crate::ros2_bridge::mapper::{Direction, TopicMapper};

use super::config::Ros2BridgeBuilder;
use super::{IntoTopicMapper, TopicQos};

/// After [`Ros2BridgeBuilder::from_ros`]: only [`FromRos::to_bus`].
pub struct FromRos {
    parent: Ros2BridgeBuilder,
    ros_topic: String,
    ros_qos: TopicQos,
}

/// After [`FromRos::to_bus`]: only [`FromRosToBus::mapper`].
pub struct FromRosToBus {
    parent: Ros2BridgeBuilder,
    ros_topic: String,
    ros_qos: TopicQos,
    bus_topic: String,
    bus_qos: TopicQos,
}

/// ROS2→bus route ready for [`.lazy()`](Ros2ToBusReady::lazy) / [`.add()`](Ros2ToBusReady::add).
pub struct Ros2ToBusReady {
    parent: Ros2BridgeBuilder,
    ros_topic: String,
    ros_qos: TopicQos,
    bus_topic: String,
    bus_qos: TopicQos,
    mapper: Arc<dyn TopicMapper>,
    lazy: bool,
}

/// After [`Ros2BridgeBuilder::from_bus`]: only [`FromBus::to_ros`].
pub struct FromBus {
    parent: Ros2BridgeBuilder,
    bus_topic: String,
    bus_qos: TopicQos,
}

/// After [`FromBus::to_ros`]: only [`FromBusToRos::mapper`].
pub struct FromBusToRos {
    parent: Ros2BridgeBuilder,
    ros_topic: String,
    ros_qos: TopicQos,
    bus_topic: String,
    bus_qos: TopicQos,
}

/// Bus→ROS2 route ready for [`.add()`](BusToRos2Ready::add).
pub struct BusToRos2Ready {
    parent: Ros2BridgeBuilder,
    ros_topic: String,
    ros_qos: TopicQos,
    bus_topic: String,
    bus_qos: TopicQos,
    mapper: Arc<dyn TopicMapper>,
}

impl Ros2BridgeBuilder {
    pub fn from_ros(self, ros_topic: impl Into<String>, ros_qos: TopicQos) -> FromRos {
        FromRos {
            parent: self,
            ros_topic: ros_topic.into(),
            ros_qos,
        }
    }

    pub fn from_bus(self, bus_topic: impl Into<String>, bus_qos: TopicQos) -> FromBus {
        FromBus {
            parent: self,
            bus_topic: bus_topic.into(),
            bus_qos,
        }
    }
}

impl FromRos {
    pub fn to_bus(self, bus_topic: impl Into<String>, bus_qos: TopicQos) -> FromRosToBus {
        FromRosToBus {
            parent: self.parent,
            ros_topic: self.ros_topic,
            ros_qos: self.ros_qos,
            bus_topic: bus_topic.into(),
            bus_qos,
        }
    }
}

impl FromRosToBus {
    pub fn mapper(self, mapper: impl IntoTopicMapper) -> Ros2ToBusReady {
        Ros2ToBusReady {
            parent: self.parent,
            ros_topic: self.ros_topic,
            ros_qos: self.ros_qos,
            bus_topic: self.bus_topic,
            bus_qos: self.bus_qos,
            mapper: mapper.into_topic_mapper(),
            lazy: false,
        }
    }
}

impl Ros2ToBusReady {
    /// Opt-in lazy ROS 2 subscription. Default is eager at `build()`.
    pub fn lazy(mut self) -> Self {
        self.lazy = true;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        self.parent.push_route(
            self.ros_topic,
            self.bus_topic,
            self.mapper,
            Direction::Ros2ToBus,
            self.lazy,
            self.ros_qos,
            self.bus_qos,
        )
    }
}

impl FromBus {
    pub fn to_ros(self, ros_topic: impl Into<String>, ros_qos: TopicQos) -> FromBusToRos {
        FromBusToRos {
            parent: self.parent,
            ros_topic: ros_topic.into(),
            ros_qos,
            bus_topic: self.bus_topic,
            bus_qos: self.bus_qos,
        }
    }
}

impl FromBusToRos {
    pub fn mapper(self, mapper: impl IntoTopicMapper) -> BusToRos2Ready {
        BusToRos2Ready {
            parent: self.parent,
            ros_topic: self.ros_topic,
            ros_qos: self.ros_qos,
            bus_topic: self.bus_topic,
            bus_qos: self.bus_qos,
            mapper: mapper.into_topic_mapper(),
        }
    }
}

impl BusToRos2Ready {
    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        self.parent.push_route(
            self.ros_topic,
            self.bus_topic,
            self.mapper,
            Direction::BusToRos2,
            false,
            self.ros_qos,
            self.bus_qos,
        )
    }
}
