# ROS 2 性能测试报告

由 `benches/ros2_perf/run.sh`（容器内 `ros2_perf_bench`）生成，方法对齐 `docs/perf-report.md`。

## 环境

- ROS: Humble (rmw_fastrtps_cpp)
- Fast DDS profile: `/root/robot-bus/benches/ros2_perf/config/fastdds_shm.xml`
- Payload: 64 bytes
- Message max loss / trial / rate range: 1% / ~1s / 500..500000 Hz (KeepLast(2048) best_effort)
- Message latency samples: 5000 (paced)
- Service/action iterations: 20000 / 5000
- Modes: **shm** (Fast DDS Shared Memory) + **udp** (Fast DDS UDPv4 only)

## 方法

- RMW: `rmw_fastrtps_cpp`；传输由 Fast DDS XML 固定为 **SHM** 或 **UDPv4**。
- 单进程多 Node + `MultiThreadedExecutor`（本机回环，非跨机）。
- Payload：64 字节；QoS `KeepLast(2048)` best_effort。
- Message **吞吐（主指标）**：按目标速率限速发送约 1s，**二分搜索**丢包率 ≤ 1% 且发送窗口内 pub/sub 均 ≥90% 目标速率的最大可持续速率（max goodput）。
- Message **延迟**：另做限速抽样（发一条等收到再发）。
- Service / action 延迟：每次 call / send_goal 本地计时。
- 指标机器相关，不作为 CI 门槛。

## 横比

message 为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。

| 场景 | shm | udp |
|------|-----|-----|
| message 发布 | 111807/s | 117330/s |
| message max goodput | 111797/s (100.0% delivered) | 117304/s (100.0% delivered) |
| service call | 19797/s | 18228/s |
| action send_goal | 1815/s | 1848/s |

## shm（Fast DDS Shared Memory）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 111807 | 111797 | 1.000s | 111807 | 111797 | 100.0 | 57 | 96 | 130 | 437 |
| service call | 20000 | 20000 | 1.010s | 19797 | 19797 | 100.0 | 47 | 72 | 102 | 50 |
| action send_goal | 5000 | 5000 | 2.754s | 1815 | 1815 | 100.0 | 543 | 905 | 1023 | 551 |

## udp（Fast DDS UDPv4，无 SHM）

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 117330 | 117304 | 1.000s | 117330 | 117304 | 100.0 | 59 | 93 | 130 | 60 |
| service call | 20000 | 20000 | 1.097s | 18228 | 18228 | 100.0 | 53 | 71 | 102 | 55 |
| action send_goal | 5000 | 5000 | 2.706s | 1848 | 1848 | 100.0 | 518 | 917 | 1057 | 541 |

## 复现

```bash
./benches/ros2_perf/run.sh
ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh
```
