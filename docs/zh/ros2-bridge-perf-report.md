[English](../en/ros2-bridge-perf-report.md) | 中文

# ROS2 Bridge性能测试报告

由 `just perf-ros2-bridge`（`ros2_bridge_perf`）生成，**不进 CI**。

## 环境

- 日期：2026-08-31。
- 本机有 `/opt/ros/humble`，但 Docker 容器 `ros2` **不存在**（`docker inspect ros2` 失败），因此默认的 `just perf-ros2-bridge`（进容器跑）无法执行。
- 本机 `source /opt/ros/humble/setup.bash` 后 `cargo check --release --bin ros2_bridge_perf --features ros2` **未通过**：typed topic mapper 与发行版 rust IDL 字段类型不一致（例如 `visualization_msgs/MenuEntry.command` 为 `std::string::String` 而非 `rosidl_runtime_rs::String`，`command_type` 为 `u8` 而非 `u32`）。无 `/tmp/ros2_rust_ws` overlay。
- 未新增 GitHub Actions Humble/Jazzy job（按计划）。
- Harness 已加 Trigger / Fibonacci 双向场景（KeepLast 与 topic 档一致）；数字需在可用的 `ros2` 容器里回填。

## 方法

- 进程内 broker + `Ros2Bridge`；ROS与 bus各一条 peer。
- 场景：64B `std_msgs/String`；Image默认 640×480 rgb8（`ROS2_BRIDGE_PERF_IMAGE_WIDTH/HEIGHT`可改）；Trigger / Fibonacci 双向。
- 方向：ROS→bus与 bus→ROS。
- 吞吐：限速发送约 1s，二分搜索丢包 ≤ 1% 的最大可持续速率。
- 延迟：另做限速抽样（发一条等收到再发）。
- QoS：KeepLast(2048) best_effort（仅 bench）。

## 结果

| scenario | sent | recv | pub/s | sub/s | delivery | p50 (µs) | p99 (µs) |
|----------|------|------|-------|-------|----------|----------|----------|
| *(未跑：缺 `ros2` 容器；本机 `--features ros2` 编不过)* | | | | | | | |
