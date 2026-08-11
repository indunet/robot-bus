//! C ABI stub: ROS 2 bridge lives in native C++ (`robot_bus_ros2_bridge` / rclcpp).
//!
//! `robot_bus_ros2_available` is deprecated and always returns 0. Prefer
//! `robot_bus::ros2_available()` which reflects compile-time `ROBOT_BUS_HAS_ROS2`.

use std::os::raw::c_int;

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_ros2_available() -> c_int {
    0
}
