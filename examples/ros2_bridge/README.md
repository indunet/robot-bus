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

## Direction in these examples

All `builtin` and `custom_add_two_ints` routes use `from_ros → to_bus`.
For topics, run a ROS publisher and a bus subscriber. For services and actions,
run the **server on bus** and the **client on ROS**: the bridge exposes the ROS
proxy that forwards requests/goals to bus. Responses/feedback/results return to ROS.
The `custom_add_two_ints` programs already host their bus server in-process.
Start one of them, then call
`ros2 service call /examples/add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 3}"`.
To call an existing ROS server from bus, reverse the chain to
`from_bus(..., TopicQos.bus()).to_ros(..., TopicQos.default())`
(C++ uses `ros_default()`).
