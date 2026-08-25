# ros2_bridge_perf

In-process **ROS 2 ↔ robot-bus** goodput / latency (not CI).

Requires sourced Humble/Jazzy **and** ament rust overlay (`share/std_msgs/rust`, `share/sensor_msgs/rust`, `share/example_interfaces/rust`). See [ros2-bridge.md](../../docs/zh/ros2-bridge.md).

```bash
just perf-ros2-bridge
# or inside a sourced overlay:
cargo run --release --bin ros2_bridge_perf --features ros2
```

Writes [`docs/zh/ros2-bridge-perf-report.md`](../../docs/zh/ros2-bridge-perf-report.md) and the English twin.

| Variable | Default | Meaning |
|----------|---------|---------|
| `ROS2_BRIDGE_PERF_IMAGE_WIDTH` / `_HEIGHT` | `640` / `480` | rgb8 Image size |
| `ROS2_BRIDGE_PERF_MAX_LOSS_PCT` | `1.0` | loss threshold (%) |
| `ROS2_BRIDGE_PERF_GOODPUT_TRIAL_SECS` | `1.0` | trial duration |
| `ROS2_BRIDGE_PERF_GOODPUT_RATE_LO` / `_HI` | `50` / `50000` | binary-search Hz |
| `ROS2_BRIDGE_PERF_MSG_LATENCY_SAMPLES` | `200` | paced latency samples |
| `ROS2_BRIDGE_PERF_ONLY` | (empty) | `string` or `image` |
