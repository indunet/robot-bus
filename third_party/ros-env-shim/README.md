# ros-env (workspace patch)

crates.io `ros-env` 0.2 with `use_ros_shim` generates an empty `interfaces.rs`.
robot-bus topic mappers still typecheck against `ros_env::sensor_msgs` / `std_msgs`
field layouts.

This path crate:

- **without `use_ros_shim`**: same overlay include as upstream `ros-env` 0.2
- **with `use_ros_shim`**: generated typed message stubs (no C typesupport) so `just check-ros2-shim` compiles topic mappers without a ROS overlay. Not a DynamicMessage fallback.

Wired via `[patch.crates-io]` in the robot-bus workspaces. Not published.
