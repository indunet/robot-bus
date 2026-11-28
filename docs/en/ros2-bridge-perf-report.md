English | [中文](../zh/ros2-bridge-perf-report.md)

# ROS 2 bridge performance report

**Bridge throughput and latency sampling is pending.** The 2026-11-20 [same-host performance comparison](performance.md) measured the native Robot Bus and ROS 2 Humble paths. A bridge run must send traffic through `Ros2Bridge` and capture each direction independently.

## Verification status

On 2026-11-20, `ros2_bridge_perf` compiled and linked against real Humble and Jazzy installations, and the bridge routes underwent functional checks. The [support status](support.md) records the Rust, ROS and system package versions and the scope of the Topic, Trigger Service and Fibonacci Action checks.

The historical 2026-08-31 run was blocked by a missing default `ros2` container and mismatches between generated mapper types and the host ROS Rust message fields. Those mismatches included the string type of `command` and the integer width of `command_type` in `visualization_msgs/MenuEntry`.

## Measurement method

- Use an in-process broker and `Ros2Bridge` with one ROS peer and one Bus peer. Measure ROS → Bus and Bus → ROS separately.
- Topic scenarios are 64 B `std_msgs/String` and 640×480 rgb8 Image by default. Service uses Trigger and Action uses Fibonacci, with both directions measured.
- Use paced throughput windows of about one second to search for a rate within 1% loss and collect separate one-at-a-time latency samples. Record QoS, actual completions for each direction and raw latency samples for each round.

Run with `just perf-ros2-bridge`. [Bridge bench](../../benches/ros2_bridge_perf/README.md)
