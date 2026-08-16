# Custom Ros2Bridge example (`my_pkg`)

A custom bridge needs **two** interface definitions with matching fields, then
a mapper that converts between them.

```text
examples/ros2_bridge/
  ros2/my_pkg/                # ROS 2 interfaces (.msg / .srv / .action)
    srv/AddTwoInts.srv       ← used by the runnable demo
    msg/Sum.msg              ← shape reference (not mounted)
    action/Compute.action    ← shape reference (not mounted)
  proto/my_pkg/              # bus protobuf (your project type, not SDK)
    srv/v1/add_two_ints.proto
```

| Side | File | Role |
|------|------|------|
| ROS | [`ros2/my_pkg/srv/AddTwoInts.srv`](ros2/my_pkg/srv/AddTwoInts.srv) | What `rclcpp` / `rclpy` / `rclrs` create |
| Bus | [`proto/my_pkg/srv/v1/add_two_ints.proto`](proto/my_pkg/srv/v1/add_two_ints.proto) | What robot-bus publishes / calls |
| Glue | `*/custom_add_two_ints.*` | Field ↔ protobuf mapper + `Ros2Bridge` mount |

Runnable programs still smoke against system
`example_interfaces/srv/AddTwoInts` (same field layout) so you can
`ros2 service call` without `colcon build` of `my_pkg`. In production both
the `.srv` and the `.proto` would be your package, and `type_name()` /
`ros_srv_type()` would point at `my_pkg/srv/AddTwoInts`.
