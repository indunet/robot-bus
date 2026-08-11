//! Executor / node control handles and callback groups.

use napi_derive::napi;
use robot_bus::runtime::{
    CallbackGroup, CallbackGroupType, NodeActionServer as RustNodeActionServer,
    NodeService as RustNodeService, ShutdownHandle as RustShutdownHandle,
    SubscriptionHandle as RustSubscriptionHandle, TimerHandle as RustTimerHandle,
};

#[napi]
#[derive(Clone)]
pub struct ShutdownHandle {
    pub(crate) inner: RustShutdownHandle,
}

#[napi]
impl ShutdownHandle {
    #[napi]
    pub fn shutdown(&self) {
        self.inner.shutdown();
    }

    #[napi]
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }
}

#[napi]
pub struct TimerHandle {
    pub(crate) inner: RustTimerHandle,
}

/// Opaque subscription id; destroy via [`crate::node::Node::destroy_subscription`].
#[napi]
pub struct SubscriptionHandle {
    pub(crate) inner: Option<RustSubscriptionHandle>,
}

#[napi]
impl SubscriptionHandle {
    #[napi(getter)]
    pub fn id(&self) -> Option<u32> {
        self.inner.map(|h| h.id() as u32)
    }
}

/// Service server handle; destroy via [`crate::node::Node::destroy_service`].
#[napi]
pub struct ServiceHandle {
    pub(crate) inner: Option<RustNodeService>,
}

#[napi]
impl ServiceHandle {
    #[napi(getter)]
    pub fn id(&self) -> Option<u32> {
        self.inner.as_ref().map(|h| h.id() as u32)
    }

    #[napi(getter)]
    pub fn service_name(&self) -> Option<String> {
        self.inner.as_ref().map(|h| h.service_name().to_string())
    }
}

/// Action server handle; destroy via [`crate::node::Node::destroy_action_server`].
#[napi]
pub struct ActionServerHandle {
    pub(crate) inner: Option<RustNodeActionServer>,
}

#[napi]
impl ActionServerHandle {
    #[napi(getter)]
    pub fn id(&self) -> Option<u32> {
        self.inner.as_ref().map(|h| h.id() as u32)
    }

    #[napi(getter)]
    pub fn action_name(&self) -> Option<String> {
        self.inner.as_ref().map(|h| h.action_name().to_string())
    }
}

#[napi]
pub enum JsCallbackGroupType {
    MutuallyExclusive = 0,
    Reentrant = 1,
}

impl From<JsCallbackGroupType> for CallbackGroupType {
    fn from(value: JsCallbackGroupType) -> Self {
        match value {
            JsCallbackGroupType::MutuallyExclusive => CallbackGroupType::MutuallyExclusive,
            JsCallbackGroupType::Reentrant => CallbackGroupType::Reentrant,
        }
    }
}

#[napi]
#[derive(Clone)]
pub struct JsCallbackGroup {
    pub(crate) inner: CallbackGroup,
}

#[napi]
impl JsCallbackGroup {
    #[napi(getter)]
    pub fn id(&self) -> u64 {
        self.inner.id()
    }

    #[napi(getter)]
    pub fn kind(&self) -> JsCallbackGroupType {
        match self.inner.kind() {
            CallbackGroupType::MutuallyExclusive => JsCallbackGroupType::MutuallyExclusive,
            CallbackGroupType::Reentrant => JsCallbackGroupType::Reentrant,
        }
    }
}
