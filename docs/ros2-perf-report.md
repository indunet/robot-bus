# ROS 2 性能测试报告

由 `benches/ros2_perf/run.sh`（容器内 `ros2_perf_bench`）生成，方法对齐 `docs/perf-report.md`。

## 环境

- ROS: Humble (rmw_fastrtps_cpp)
- Fast DDS profile: `/root/robot-bus/benches/ros2_perf/config/fastdds_shm.xml`
- Payload: 64 bytes
- Message max loss / trial / rate range: 1% / ~1s / 500..500000 Hz (KeepLast(2048) best_effort)
- Message latency samples: 5000 (paced)
- Service/action iterations: 50000 / 5000
- Modes: **shm** (Fast DDS Shared Memory) + **udp** (Fast DDS UDPv4 only)

## 方法

- RMW: `rmw_fastrtps_cpp`；传输由 Fast DDS XML 固定为 **SHM** 或 **UDPv4**。
- 单进程多 Node + `MultiThreadedExecutor`（本机回环，非跨机）。
- Payload：64 字节；QoS `KeepLast(2048)` best_effort。
- Message **吞吐（主指标）**：在目标速率下限速发送，**二分搜索**丢包率 ≤ 1% 的最大可持续速率（max goodput）；每档约 1s。
- Message **延迟**：另做限速抽样（发一条等收到再发）。
- Service / action 延迟：每次 call / send_goal 本地计时。
- 指标机器相关，不作为 CI 门槛。

## 横比

message 为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。

| 场景 | shm | udp |
|------|-----|-----|
| message 发布 | 120265/s | 108731/s |
| message max goodput | 120265/s (100.0% delivered) | 108731/s (100.0% delivered) |
| service call | 19439/s | 19688/s |
| action send_goal | 1885/s | 1887/s |

## shm（Fast DDS Shared Memory）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 50000 | 50000 | 0.416s | 120265 | 120265 | 100.0 | 57 | 98 | 156 | 61 |
| service call | 50000 | 50000 | 2.572s | 19439 | 19439 | 100.0 | 48 | 73 | 103 | 51 |
| action send_goal | 5000 | 5000 | 2.653s | 1885 | 1885 | 100.0 | 527 | 881 | 969 | 531 |

## udp（Fast DDS UDPv4，无 SHM）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|


## shm（Fast DDS Shared Memory）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|

## udp（Fast DDS UDPv4，无 SHM）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 50000 | 50000 | 0.460s | 108731 | 108731 | 100.0 | 58 | 96 | 131 | 60 |
| service call | 50000 | 50000 | 2.540s | 19688 | 19688 | 100.0 | 48 | 70 | 100 | 51 |
| action send_goal | 5000 | 5000 | 2.650s | 1887 | 1887 | 100.0 | 531 | 855 | 940 | 530 |

## 复现

```bash
./benches/ros2_perf/run.sh
ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh
```
