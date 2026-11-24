# robot-bus

ZeroMQ message bus for Rust: the participant SDK (`Node`, executors, and topic, service, and action clients) and the broker (`RobotBusBroker`), WebSocket server, embedded console, and `rbus` CLI, in one crate.

```toml
robot-bus = "3"
```

The default `full` feature set enables `broker`, `ws`, `console`, and `demo-tank`. Programs that only create `Node`s can opt out:

```toml
robot-bus = { version = "3", default-features = false }
```

Generated messages live in `robot-bus-msgs` and are re-exported here. The ROS 2 bridge is the separate `robot-bus-ros2` crate and is not published.
