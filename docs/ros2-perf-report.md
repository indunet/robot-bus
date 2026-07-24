# ROS 2 性能测试报告

由 `benches/ros2_perf/run.sh`（容器内 `ros2_perf_bench`）生成，方法对齐 `docs/perf-report.md`。

## 环境

- ROS: Humble (rmw_fastrtps_cpp)
- Fast DDS profile: `/tmp/ros2_perf_ws/ros2_perf/config/fastdds_shm.xml`
- Payload: 64 bytes
- Message iters / recv target / history: 100000 / 80000 / KeepLast(100000) best_effort
- Message latency samples: 5000 (paced)
- Service/action iterations: 100000 / 100000
- Modes: **shm** (Fast DDS Shared Memory) + **udp** (Fast DDS UDPv4 only)

## 方法

- RMW: `rmw_fastrtps_cpp`；传输由 Fast DDS XML 固定为 **SHM** 或 **UDPv4**。
- 单进程多 Node + `MultiThreadedExecutor`（本机回环，非跨机）。
- Payload：64 字节；QoS `KeepLast(100000)` best_effort。
- Message **吞吐**：尽快发送 N=100000（KeepLast=100000 best_effort），订阅端收到 ≥80000 即结束统计。
- Message **延迟**：另做限速抽样（发一条等收到再发）。
- Service / action 延迟：每次 call / send_goal 本地计时。
- 指标机器相关，不作为 CI 门槛。

## 横比

message 为 **订阅速率（投递率）**；另附发布速率。

| 场景 | shm | udp |
|------|-----|-----|
| message 发布 | 832/s | 832/s |
| message 订阅 | 86/s (10.4%) | 113/s (13.6%) |
| service call | — | — |
| action send_goal | — | — |

## shm（Fast DDS Shared Memory）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 10381 | 120.223s | 832 | 86 | 10.4 | 67 | 184 | 263 | 90 |

## udp（Fast DDS UDPv4，无 SHM）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|


## shm（Fast DDS Shared Memory）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|

## udp（Fast DDS UDPv4，无 SHM）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 13574 | 120.228s | 832 | 113 | 13.6 | 109 | 220 | 348 | 117 |

## 复现

```bash
./benches/ros2_perf/run.sh
ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh
```
