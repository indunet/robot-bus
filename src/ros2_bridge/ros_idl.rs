//! Ament-re-exported ROS IDL (`ros-env`) plus shim stubs for `ros2-shim`.
//!
//! Real `--features ros2` builds expect overlay crates on `AMENT_PREFIX_PATH`
//! (`share/<pkg>/rust/`). `just check-ros2-shim` uses the field stubs below so
//! conversion unit tests still compile without an overlay.

#![allow(non_camel_case_types)]

#[cfg(not(feature = "ros2-shim"))]
pub use ros_env::example_interfaces;

#[cfg(feature = "ros2-shim")]
#[allow(dead_code)]
pub mod example_interfaces {
    pub mod action {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Fibonacci_Goal {
            pub order: i32,
        }

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Fibonacci_Feedback {
            pub sequence: Vec<i32>,
        }

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Fibonacci_Result {
            pub sequence: Vec<i32>,
        }

        pub struct Fibonacci;
    }

    pub mod srv {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct AddTwoInts_Request {
            pub a: i64,
            pub b: i64,
        }

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct AddTwoInts_Response {
            pub sum: i64,
        }

        pub struct AddTwoInts;
    }
}
