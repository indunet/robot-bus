//! Builtin [`ServiceMapper`] type tags (Trigger / SetBool).
//!
//! Field converters: [`super::service`]. Library wiring: [`crate::ros2_bridge::typed_rpc`].

use std::sync::Arc;

use crate::errors::{BusError, Result};
use crate::ros2_bridge::mapper::ServiceMapper;

/// Builtin codec tag for `std_srvs/srv/Trigger`.
pub struct TriggerServiceMapper;
/// Builtin codec tag for `std_srvs/srv/SetBool`.
pub struct SetBoolServiceMapper;

pub fn lookup_service_mapper(type_name: &str) -> Result<Arc<dyn ServiceMapper>> {
    match type_name {
        "std_srvs/srv/Trigger" => Ok(Arc::new(TriggerServiceMapper)),
        "std_srvs/srv/SetBool" => Ok(Arc::new(SetBoolServiceMapper)),
        other => Err(BusError::Protocol(format!(
            "unsupported ros2 bridge service type {other:?}; \
             builtins: std_srvs/srv/Trigger, std_srvs/srv/SetBool; \
             for a custom Rust typed backend implement ServiceMapper::attach; \
             arbitrary codecs need dynamic service support (Track B)"
        ))),
    }
}

impl ServiceMapper for TriggerServiceMapper {
    fn type_name(&self) -> &'static str {
        "std_srvs/srv/Trigger"
    }
}

impl ServiceMapper for SetBoolServiceMapper {
    fn type_name(&self) -> &'static str {
        "std_srvs/srv/SetBool"
    }
}
