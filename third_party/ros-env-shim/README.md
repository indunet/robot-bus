# ros-env (workspace patch)

crates.io `ros-env` 0.2 with `use_ros_shim` generates an empty `interfaces.rs`.
rclrs 0.8 (git) still typechecks against `ros_env::action_msgs` / `builtin_interfaces` /
`rcl_interfaces` field layouts.

This path crate:

- **without `use_ros_shim`**: same overlay include as upstream `ros-env` 0.2
- **with `use_ros_shim`**: generated typed message stubs (no C typesupport) so `just check-ros2-shim` compiles topic mappers without a ROS overlay. Not a DynamicMessage fallback.

Wired via `[patch.crates-io]` in the robot-bus workspaces. Not published.
