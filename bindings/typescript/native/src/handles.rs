//! Executor / node control handles and callback groups.

use napi_derive::napi;
use robot_bus::runtime::{
    CallbackGroup, CallbackGroupType, ShutdownHandle as RustShutdownHandle,
    TimerHandle as RustTimerHandle,
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
