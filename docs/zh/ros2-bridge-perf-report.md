[English](../en/ros2-bridge-perf-report.md) | 中文

# ROS 2 桥接性能测试报告

**桥接吞吐和延迟待采样。** 2026-11-20 的同机复测分别测量了 [Robot Bus](perf-report.md) 和 [ROS 2 Humble](ros2-perf-report.md) 的独立路径。桥接路径需要让流量实际经过 `Ros2Bridge`，并单独记录两方向的数据。

## 当前验证状态

2026-11-20 已在真实 Humble 和 Jazzy 环境编译并链接 `ros2_bridge_perf`，并进行了桥接路由功能验证。[支持状态](support.md)记录了所用的 Rust、ROS 和系统包版本，以及 Topic、Trigger Service 和 Fibonacci Action 的验证范围。

2026-08-31 的历史运行因默认容器 `ros2` 缺失，以及宿主机 ROS Rust 消息字段与 mapper 类型不一致而未能采样。其中包括 `visualization_msgs/MenuEntry` 的 `command` 字符串类型和 `command_type` 整数宽度差异。

## 采样方法

- 进程内 broker 和 `Ros2Bridge`，ROS 与 Bus 各一个 peer，分别运行 ROS → Bus 和 Bus → ROS。
- 话题场景为 64 B `std_msgs/String` 和默认 640×480 rgb8 Image。Service 为 Trigger，Action 为 Fibonacci，两者均测双向。
- 消息以约 1 秒的限速窗口搜索满足丢失率不超过 1% 的速率，延迟另做逐条单向采样。每轮记录 QoS、各方向实际完成数和原始延迟样本。

运行入口为 `just perf-ros2-bridge`。 [桥接 bench](../../benches/ros2_bridge_perf/README.md) · [性能说明](performance.md)
