//! Chained builder for [`Ros2Bridge`].
//!
//! Configuration is code-only: attach concrete mapper objects via `.mapper(...)`.
//! There is no YAML loader and no type-name-string route API.

mod bridge;
mod config;
mod rpc;
mod specs;
mod topic;
mod wire;

#[cfg(test)]
mod route_mapper_tests;

use std::sync::Arc;

use super::mapper::{ActionMapper, ServiceMapper, TopicMapper};

pub use bridge::Ros2Bridge;
pub use config::Ros2BridgeBuilder;
pub use rpc::{
    Action, ActionFromBus, ActionFromRos, ActionPair, ActionReady, Service, ServiceFromBus,
    ServiceFromRos, ServicePair, ServiceReady,
};
pub use specs::{ACTION_CALL_TIMEOUT, SERVICE_CALL_TIMEOUT};
pub use topic::{BusToRos2Ready, FromBus, FromBusToRos, FromRos, FromRosToBus, Ros2ToBusReady};

pub use super::mapper::TopicQos;

/// Console / log label: `type_name()` if set, else the Rust type's last path segment.
pub(crate) fn topic_mapper_label(mapper: &dyn TopicMapper, rust_name: &str) -> String {
    let n = mapper.type_name();
    if !n.is_empty() {
        return n.to_string();
    }
    rust_name
        .rsplit("::")
        .next()
        .unwrap_or(rust_name)
        .trim_end_matches(['>', ' '])
        .to_string()
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
