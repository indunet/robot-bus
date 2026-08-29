[English](../en/ros2-bridge-perf-report.md) | 中文

# ROS2 Bridge性能测试报告

由 `just perf-ros2-bridge`（`ros2_bridge_perf`）生成，**不进 CI**。尚未在本机跑过时，表格为空。

## 环境

- 运行 `just perf-ros2-bridge`（`ros2`容器）后回填。

## 方法

- 进程内 broker + `Ros2Bridge`；ROS与 bus各一条 peer。
- 场景：64B `std_msgs/String`；Image默认 640×480 rgb8（`ROS2_BRIDGE_PERF_IMAGE_WIDTH/HEIGHT`可改）。
- 方向：ROS→bus与 bus→ROS。
- 吞吐：限速发送约 1s，二分搜索丢包 ≤ 1% 的最大可持续速率。
- 延迟：另做限速抽样（发一条等收到再发）。
- QoS：KeepLast(2048) best_effort（仅 bench）。

## 结果

| scenario | sent | recv | pub/s | sub/s | delivery | p50 (µs) | p99 (µs) |
|----------|------|------|-------|-------|----------|----------|----------|
| *(run `just perf-ros2-bridge`)* | | | | | | | |
