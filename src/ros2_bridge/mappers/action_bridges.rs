//! Builtin [`ActionMapper`] type tag (Fibonacci).
//!
//! Field converters: [`super::action`]. Library wiring: [`crate::ros2_bridge::typed_rpc`].

use std::sync::Arc;

use crate::errors::{BusError, Result};
use crate::ros2_bridge::mapper::ActionMapper;

/// Builtin codec tag for `example_interfaces/action/Fibonacci`.
pub struct FibonacciActionMapper;

pub fn lookup_action_mapper(type_name: &str) -> Result<Arc<dyn ActionMapper>> {
    match type_name {
        "example_interfaces/action/Fibonacci" => Ok(Arc::new(FibonacciActionMapper)),
        other => Err(BusError::Protocol(format!(
            "unsupported ros2 bridge action type {other:?}; \
             builtin: example_interfaces/action/Fibonacci; \
             for a custom Rust typed backend implement ActionMapper::attach; \
             arbitrary codecs need dynamic action support (Track B)"
        ))),
    }
}

impl ActionMapper for FibonacciActionMapper {
    fn type_name(&self) -> &'static str {
        "example_interfaces/action/Fibonacci"
    }
}
