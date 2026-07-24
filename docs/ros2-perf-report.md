# ROS 2 性能测试报告

由 `benches/ros2_perf/run.sh`（容器内 `ros2_perf_bench`）生成，方法对齐 `docs/perf-report.md`。

## 环境

- ROS: Humble (rmw_fastrtps_cpp)
- Fast DDS profile: shm → `fastdds_shm.xml`；udp → `fastdds_udp.xml`
- Payload: 64 bytes
- Message max loss / trial / rate range: 1% / ~1s (cap 50000 msgs) / 500..500000 Hz (KeepLast(2048) best_effort)
- Message latency samples: 5000 (paced)
- Service/action iterations: 100000 / 100000
- Modes: **shm** (Fast DDS Shared Memory) + **udp** (Fast DDS UDPv4 only)
- Host: Linux container `ros2`（Docker Desktop VM）；与 macOS 上 `just perf` 不在同一 OS

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
| message 发布 | 74355/s | 99810/s |
| message max goodput | 74355/s (100.0% delivered) | 99810/s (100.0% delivered) |
| service call | 16395/s | 27497/s |
| action send_goal | 128/s | 135/s |

## shm（Fast DDS Shared Memory）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 50000 | 50000 | 0.672s | 74355 | 74355 | 100.0 | 32 | 56 | 78 | 37 |
| service call | 100000 | 100000 | 6.099s | 16395 | 16395 | 100.0 | 58 | 90 | 118 | 61 |
| action send_goal | 100000 | 100000 | 780.453s | 128 | 128 | 100.0 | 7196 | 15085 | 19414 | 7804 |

## udp（Fast DDS UDPv4，无 SHM）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 50000 | 50000 | 0.501s | 99810 | 99810 | 100.0 | 85 | 210 | 497 | 107 |
| service call | 100000 | 100000 | 3.637s | 27497 | 27497 | 100.0 | 37 | 45 | 54 | 36 |
| action send_goal | 100000 | 100000 | 743.262s | 135 | 135 | 100.0 | 7419 | 14091 | 16249 | 7432 |

## 复现

```bash
./benches/ros2_perf/run.sh
ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh
```
