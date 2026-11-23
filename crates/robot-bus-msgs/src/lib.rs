//! Generated ROS 2–style protobuf messages and the service/action marker traits.
//!
//! On-disk stubs live under `src/generated/` (gitignored; regenerate with
//! `just gen-rust`). Packages are re-exported at this crate's root and again
//! from `robot-bus`, so existing `robot_bus::<pkg>::…` paths stay valid.

mod generated;
pub mod typed;

pub use generated::{
    ackermann_msgs, action, action_msgs, apriltag_msgs, builtin_interfaces, control_msgs,
    diagnostic_msgs, example_interfaces, foxglove_msgs, geometry_msgs, lifecycle_msgs, map_msgs,
    nav2_msgs, nav_msgs, robot_bus_interfaces, sensor_msgs, shape_msgs, std_msgs, std_srvs,
    stereo_msgs, tf2_msgs, trajectory_msgs, unique_identifier_msgs, vision_msgs,
    visualization_msgs,
};
pub use typed::{Action, ActionOutcome, Service};
