use std::sync::Arc;
use std::time::Duration;

use crate::errors::Result;
use crate::ros2_bridge::mapper::{ActionMapper, Direction, ServiceMapper};

use super::config::Ros2BridgeBuilder;
use super::specs::{ACTION_CALL_TIMEOUT, SERVICE_CALL_TIMEOUT};
use super::{IntoActionMapper, IntoServiceMapper, TopicQos};

/// After [`Ros2BridgeBuilder::service`]: only [`Service::from_ros`] / [`Service::from_bus`].
pub struct Service {
    parent: Ros2BridgeBuilder,
}

/// After [`Service::from_ros`]: only [`ServiceFromRos::to_bus`].
pub struct ServiceFromRos {
    parent: Ros2BridgeBuilder,
    ros_service: String,
    ros_qos: TopicQos,
}

/// After [`Service::from_bus`]: only [`ServiceFromBus::to_ros`].
pub struct ServiceFromBus {
    parent: Ros2BridgeBuilder,
    bus_service: String,
    bus_qos: TopicQos,
}

/// After both service names: only [`ServicePair::mapper`].
pub struct ServicePair {
    parent: Ros2BridgeBuilder,
    ros_service: String,
    bus_service: String,
    ros_qos: TopicQos,
    bus_qos: TopicQos,
    direction: Direction,
}

/// After mapper: [`ServiceReady::timeout`] / [`ServiceReady::add`].
pub struct ServiceReady {
    parent: Ros2BridgeBuilder,
    ros_service: String,
    bus_service: String,
    mapper: Arc<dyn ServiceMapper>,
    direction: Direction,
    timeout: Duration,
    ros_qos: TopicQos,
    bus_qos: TopicQos,
}

/// After [`Ros2BridgeBuilder::action`]: only [`Action::from_ros`] / [`Action::from_bus`].
pub struct Action {
    parent: Ros2BridgeBuilder,
}

/// After [`Action::from_ros`]: only [`ActionFromRos::to_bus`].
pub struct ActionFromRos {
    parent: Ros2BridgeBuilder,
    ros_action: String,
    ros_qos: TopicQos,
}

/// After [`Action::from_bus`]: only [`ActionFromBus::to_ros`].
pub struct ActionFromBus {
    parent: Ros2BridgeBuilder,
    bus_action: String,
    bus_qos: TopicQos,
}

/// After both action names: only [`ActionPair::mapper`].
pub struct ActionPair {
    parent: Ros2BridgeBuilder,
    ros_action: String,
    bus_action: String,
    ros_qos: TopicQos,
    bus_qos: TopicQos,
    direction: Direction,
}

/// After mapper: [`ActionReady::timeout`] / [`ActionReady::add`].
pub struct ActionReady {
    parent: Ros2BridgeBuilder,
    ros_action: String,
    bus_action: String,
    mapper: Arc<dyn ActionMapper>,
    direction: Direction,
    timeout: Duration,
    ros_qos: TopicQos,
    bus_qos: TopicQos,
}

impl Ros2BridgeBuilder {
    pub fn service(self) -> Service {
        Service { parent: self }
    }

    pub fn action(self) -> Action {
        Action { parent: self }
    }

    /// Add a service route with an explicit [`ServiceMapper`].
    pub fn add_service_mapper(
        self,
        ros_service: impl Into<String>,
        bus_service: impl Into<String>,
        mapper: impl IntoServiceMapper,
        direction: Direction,
        timeout: Duration,
        ros_qos: TopicQos,
        bus_qos: TopicQos,
    ) -> Result<Self> {
        self.push_service(
            ros_service.into(),
            bus_service.into(),
            mapper.into_service_mapper(),
            direction,
            timeout,
            ros_qos,
            bus_qos,
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
        ros_qos: TopicQos,
        bus_qos: TopicQos,
    ) -> Result<Self> {
        self.push_action(
            ros_action.into(),
            bus_action.into(),
            mapper.into_action_mapper(),
            direction,
            timeout,
            ros_qos,
            bus_qos,
        )
    }
}

impl Service {
    pub fn from_ros(self, ros_service: impl Into<String>, ros_qos: TopicQos) -> ServiceFromRos {
        ServiceFromRos {
            parent: self.parent,
            ros_service: ros_service.into(),
            ros_qos,
        }
    }

    pub fn from_bus(self, bus_service: impl Into<String>, bus_qos: TopicQos) -> ServiceFromBus {
        ServiceFromBus {
            parent: self.parent,
            bus_service: bus_service.into(),
            bus_qos,
        }
    }
}

impl ServiceFromRos {
    pub fn to_bus(self, bus_service: impl Into<String>, bus_qos: TopicQos) -> ServicePair {
        ServicePair {
            parent: self.parent,
            ros_service: self.ros_service,
            bus_service: bus_service.into(),
            ros_qos: self.ros_qos,
            bus_qos,
            direction: Direction::Ros2ToBus,
        }
    }
}

impl ServiceFromBus {
    pub fn to_ros(self, ros_service: impl Into<String>, ros_qos: TopicQos) -> ServicePair {
        ServicePair {
            parent: self.parent,
            ros_service: ros_service.into(),
            bus_service: self.bus_service,
            ros_qos,
            bus_qos: self.bus_qos,
            direction: Direction::BusToRos2,
        }
    }
}

impl ServicePair {
    pub fn mapper(self, mapper: impl IntoServiceMapper) -> ServiceReady {
        ServiceReady {
            parent: self.parent,
            ros_service: self.ros_service,
            bus_service: self.bus_service,
            mapper: mapper.into_service_mapper(),
            direction: self.direction,
            timeout: SERVICE_CALL_TIMEOUT,
            ros_qos: self.ros_qos,
            bus_qos: self.bus_qos,
        }
    }
}

impl ServiceReady {
    /// Override the default service call timeout ([`SERVICE_CALL_TIMEOUT`]).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        self.parent.push_service(
            self.ros_service,
            self.bus_service,
            self.mapper,
            self.direction,
            self.timeout,
            self.ros_qos,
            self.bus_qos,
        )
    }
}

impl Action {
    pub fn from_ros(self, ros_action: impl Into<String>, ros_qos: TopicQos) -> ActionFromRos {
        ActionFromRos {
            parent: self.parent,
            ros_action: ros_action.into(),
            ros_qos,
        }
    }

    pub fn from_bus(self, bus_action: impl Into<String>, bus_qos: TopicQos) -> ActionFromBus {
        ActionFromBus {
            parent: self.parent,
            bus_action: bus_action.into(),
            bus_qos,
        }
    }
}

impl ActionFromRos {
    pub fn to_bus(self, bus_action: impl Into<String>, bus_qos: TopicQos) -> ActionPair {
        ActionPair {
            parent: self.parent,
            ros_action: self.ros_action,
            bus_action: bus_action.into(),
            ros_qos: self.ros_qos,
            bus_qos,
            direction: Direction::Ros2ToBus,
        }
    }
}

impl ActionFromBus {
    pub fn to_ros(self, ros_action: impl Into<String>, ros_qos: TopicQos) -> ActionPair {
        ActionPair {
            parent: self.parent,
            ros_action: ros_action.into(),
            bus_action: self.bus_action,
            ros_qos,
            bus_qos: self.bus_qos,
            direction: Direction::BusToRos2,
        }
    }
}

impl ActionPair {
    pub fn mapper(self, mapper: impl IntoActionMapper) -> ActionReady {
        ActionReady {
            parent: self.parent,
            ros_action: self.ros_action,
            bus_action: self.bus_action,
            mapper: mapper.into_action_mapper(),
            direction: self.direction,
            timeout: ACTION_CALL_TIMEOUT,
            ros_qos: self.ros_qos,
            bus_qos: self.bus_qos,
        }
    }
}

impl ActionReady {
    /// Override the default action goal timeout ([`ACTION_CALL_TIMEOUT`]).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn add(self) -> Result<Ros2BridgeBuilder> {
        self.parent.push_action(
            self.ros_action,
            self.bus_action,
            self.mapper,
            self.direction,
            self.timeout,
            self.ros_qos,
            self.bus_qos,
        )
    }
}
