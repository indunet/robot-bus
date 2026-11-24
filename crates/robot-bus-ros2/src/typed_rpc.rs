//! Builtin attach helpers (thin wrappers over [`TypedServiceMapper`] / [`TypedActionMapper`]).
//!
//! Prefer implementing convert methods on the mapper types; these entry points remain for
//! [`typed_service`](crate::typed_service) and the default [`ServiceMapper::attach`]
//! type-name dispatch.

use robot_bus::errors::{BusError, Result};
use crate::mapper::{ActionWireContext, ServiceWireContext};
#[cfg(feature = "demo")]
use crate::mappers::action_bridges::FibonacciActionMapper;
use crate::mappers::service_bridges::{SetBoolServiceMapper, TriggerServiceMapper};
#[cfg(feature = "demo")]
use crate::typed_wire::wire_typed_action;
use crate::typed_wire::wire_typed_service;

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
    #[cfg(feature = "demo")]
    if type_name == "example_interfaces/action/Fibonacci" {
        return attach_fibonacci(ctx);
    }
    let _ = ctx;
    #[cfg(feature = "demo")]
    let builtins = "example_interfaces/action/Fibonacci";
    #[cfg(not(feature = "demo"))]
    let builtins = "none (enable the `demo` feature for example_interfaces/action/Fibonacci)";
    Err(BusError::Protocol(format!(
        "no typed action backend for {type_name:?}; \
         builtin: {builtins}; \
         implement TypedActionMapper (convert methods) or override ActionMapper::attach"
    )))
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
#[cfg(feature = "demo")]
pub fn attach_fibonacci(ctx: ActionWireContext<'_>) -> Result<()> {
    wire_typed_action(&FibonacciActionMapper, ctx)
}
