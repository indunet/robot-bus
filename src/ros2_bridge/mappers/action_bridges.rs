//! Builtin [`TypedActionMapper`] codec (Fibonacci).

use prost::Message as ProstMessage;
use rclrs::vendor::example_interfaces::action as ros_act;

use crate::action::v1::{
    FibonacciFeedback as BusFibonacciFeedback, FibonacciGoal as BusFibonacciGoal,
    FibonacciResult as BusFibonacciResult,
};
use crate::errors::{BusError, Result};
use crate::ros2_bridge::mapper::TypedActionMapper;
use crate::ros2_bridge::mappers::action;

/// Builtin codec for `example_interfaces/action/Fibonacci`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FibonacciActionMapper;

impl TypedActionMapper for FibonacciActionMapper {
    type Ros = ros_act::Fibonacci;

    fn type_name(&self) -> &str {
        "example_interfaces/action/Fibonacci"
    }

    fn ros_goal_to_bus(&self, goal: &ros_act::Fibonacci_Goal) -> Result<Vec<u8>> {
        Ok(action::fibonacci_ros_goal_to_bus(goal).encode_to_vec())
    }

    fn bus_goal_to_ros(&self, payload: &[u8]) -> Result<ros_act::Fibonacci_Goal> {
        let bus = BusFibonacciGoal::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode FibonacciGoal: {e}")))?;
        Ok(action::fibonacci_bus_goal_to_ros(&bus))
    }

    fn ros_feedback_to_bus(&self, feedback: &ros_act::Fibonacci_Feedback) -> Result<Vec<u8>> {
        Ok(action::fibonacci_ros_feedback_to_bus(feedback).encode_to_vec())
    }

    fn bus_feedback_to_ros(&self, payload: &[u8]) -> Result<ros_act::Fibonacci_Feedback> {
        let bus = BusFibonacciFeedback::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode FibonacciFeedback: {e}")))?;
        Ok(action::fibonacci_bus_feedback_to_ros(&bus))
    }

    fn ros_result_to_bus(&self, result: &ros_act::Fibonacci_Result) -> Result<Vec<u8>> {
        Ok(action::fibonacci_ros_result_to_bus(result).encode_to_vec())
    }

    fn bus_result_to_ros(&self, payload: &[u8]) -> Result<ros_act::Fibonacci_Result> {
        let bus = BusFibonacciResult::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode FibonacciResult: {e}")))?;
        Ok(action::fibonacci_bus_result_to_ros(&bus))
    }
}
