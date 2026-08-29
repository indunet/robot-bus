//! Builtin attach helpers (thin wrappers over [`TypedServiceMapper`] / [`TypedActionMapper`]).
//!
//! Prefer implementing convert methods on the mapper types; these entry points remain for
//! [`typed_service`](crate::ros2_bridge::typed_service) and the default [`ServiceMapper::attach`]
//! type-name dispatch.

use crate::errors::{BusError, Result};
use crate::ros2_bridge::mapper::{ActionWireContext, ServiceWireContext};
use crate::ros2_bridge::mappers::service_bridges::{SetBoolServiceMapper, TriggerServiceMapper};
use crate::ros2_bridge::typed_wire::wire_typed_service;

#[cfg(not(feature = "ros2-shim"))]
use crate::ros2_bridge::mappers::action_bridges::FibonacciActionMapper;
#[cfg(not(feature = "ros2-shim"))]
use crate::ros2_bridge::typed_wire::wire_typed_action;

/// Dispatch builtin service backends by ROS type string.
pub fn attach_builtin_service(type_name: &str, ctx: ServiceWireContext<'_>) -> Result<()> {
    match type_name {
        "std_srvs/srv/Trigger" => attach_trigger(ctx),
        "std_srvs/srv/SetBool" => attach_set_bool(ctx),
        other => Err(BusError::Protocol(format!(
            "no typed service backend for {other:?}; \
             builtins: std_srvs/srv/Trigger, std_srvs/srv/SetBool; \
             implement TypedServiceMapper (convert methods) or override ServiceMapper::attach"
        ))),
    }
}

/// Dispatch builtin action backends by ROS type string.
pub fn attach_builtin_action(type_name: &str, ctx: ActionWireContext<'_>) -> Result<()> {
    match type_name {
        "example_interfaces/action/Fibonacci" => attach_fibonacci(ctx),
        other => Err(BusError::Protocol(format!(
            "no typed action backend for {other:?}; \
             builtin: example_interfaces/action/Fibonacci; \
             implement TypedActionMapper (convert methods) or override ActionMapper::attach"
        ))),
    }
}

/// Wire `std_srvs/srv/Trigger`.
pub fn attach_trigger(ctx: ServiceWireContext<'_>) -> Result<()> {
    wire_typed_service(&TriggerServiceMapper, ctx)
}

/// Wire `std_srvs/srv/SetBool`.
pub fn attach_set_bool(ctx: ServiceWireContext<'_>) -> Result<()> {
    wire_typed_service(&SetBoolServiceMapper, ctx)
}

/// Wire `example_interfaces/action/Fibonacci`.
pub fn attach_fibonacci(ctx: ActionWireContext<'_>) -> Result<()> {
    #[cfg(feature = "ros2-shim")]
    {
        let _ = ctx;
        return Err(BusError::Protocol(
            "example_interfaces/action/Fibonacci is unavailable with feature ros2-shim"
                .into(),
        ));
    }
    #[cfg(not(feature = "ros2-shim"))]
    wire_typed_action(&FibonacciActionMapper, ctx)
}
