//! Typed Fibonacci action mapping used by Ros2Bridge actions.

// --- Fibonacci action conversions (rclrs vendor ↔ bus prost) ---

use crate::action::v1::{
    FibonacciFeedback as BusFibonacciFeedback, FibonacciGoal as BusFibonacciGoal,
    FibonacciResult as BusFibonacciResult,
};
use rclrs::vendor::example_interfaces::action as ros_act;

pub fn fibonacci_ros_goal_to_bus(goal: &ros_act::Fibonacci_Goal) -> BusFibonacciGoal {
    BusFibonacciGoal { order: goal.order }
}

pub fn fibonacci_bus_goal_to_ros(goal: &BusFibonacciGoal) -> ros_act::Fibonacci_Goal {
    ros_act::Fibonacci_Goal { order: goal.order }
}

pub fn fibonacci_ros_feedback_to_bus(fb: &ros_act::Fibonacci_Feedback) -> BusFibonacciFeedback {
    BusFibonacciFeedback {
        sequence: fb.sequence.clone(),
    }
}

pub fn fibonacci_bus_feedback_to_ros(fb: &BusFibonacciFeedback) -> ros_act::Fibonacci_Feedback {
    ros_act::Fibonacci_Feedback {
        sequence: fb.sequence.clone(),
    }
}

pub fn fibonacci_ros_result_to_bus(res: &ros_act::Fibonacci_Result) -> BusFibonacciResult {
    BusFibonacciResult {
        sequence: res.sequence.clone(),
    }
}

pub fn fibonacci_bus_result_to_ros(res: &BusFibonacciResult) -> ros_act::Fibonacci_Result {
    ros_act::Fibonacci_Result {
        sequence: res.sequence.clone(),
    }
}


#[cfg(test)]
mod action_convert_tests {
    use super::*;

    #[test]
    fn fibonacci_goal_feedback_result_roundtrip() {
        let ros_goal = ros_act::Fibonacci_Goal { order: 5 };
        let bus_goal = fibonacci_ros_goal_to_bus(&ros_goal);
        assert_eq!(bus_goal.order, 5);
        assert_eq!(fibonacci_bus_goal_to_ros(&bus_goal).order, 5);

        let ros_fb = ros_act::Fibonacci_Feedback {
            sequence: vec![0, 1, 1, 2],
        };
        let bus_fb = fibonacci_ros_feedback_to_bus(&ros_fb);
        assert_eq!(bus_fb.sequence, vec![0, 1, 1, 2]);
        assert_eq!(
            fibonacci_bus_feedback_to_ros(&bus_fb).sequence,
            vec![0, 1, 1, 2]
        );

        let ros_res = ros_act::Fibonacci_Result {
            sequence: vec![0, 1, 1, 2, 3],
        };
        let bus_res = fibonacci_ros_result_to_bus(&ros_res);
        assert_eq!(bus_res.sequence, vec![0, 1, 1, 2, 3]);
        assert_eq!(
            fibonacci_bus_result_to_ros(&bus_res).sequence,
            vec![0, 1, 1, 2, 3]
        );
    }
}
