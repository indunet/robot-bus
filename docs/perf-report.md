# robot-bus 性能测试报告

由 `just perf`（`cargo run --release --bin robot_bus_perf`）生成。

## 环境

- robot-bus: 0.0.7
- 系统: macOS 26.5.2 (25F84)
- 逻辑 CPU: 10
- CPU: Apple M5
- CPU 核心(物理): 10
- CPU 核心(逻辑): 10
- 内存: 32.0 GiB

## 方法

- 进程内 `RobotBusBroker`，`bind_all_transports = true`（tcp + ipc + inproc + grpc）。
- Console HTTP 关闭；**message HWM=100000**（仅 bench，≥ 吞吐目标条数）；service/action HWM=64。
- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。
- Message **吞吐**（深队列横比）：尽快发送 N=100000（HWM=100000），**订阅端收到 ≥80000 即结束统计**；ZMQ PUB / ROS2 均为 best-effort。
- Message **延迟**：另做 5000 次限速抽样（发一条等收到再发），测单程时延。
- Service / action：各 100000 次；延迟为每次 call / send_goal 本地计时。
- ZMQ：共享 `Context` + `Node::tcp` / `ipc` / `inproc`；gRPC：`Node::grpc_at`。
- inproc 与嵌入式 broker 必须共用同一 `Context`（ZeroMQ inproc 是 context-local）。
- 指标为单机本机回环，机器相关，不作为 CI 门槛。

## 横比

message 单元格为 **订阅速率（投递率）**；service/action 为完成速率。gRPC Node **不支持 publish**，发布格为 —。

| 场景 | tcp | ipc | inproc | grpc |
|------|-----|-----|--------|------|
| message 发布 | 76762/s | 79949/s | 78785/s | — |
| message 订阅 | 61441/s (80%) | 63984/s (80%) | 63044/s (80%) | 36200/s (80%) |
| service call | — | — | — | — |
| action send_goal | — | — | — | — |

## tcp

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 80040 | 1.303s | 76762 | 61441 | 80.0 | 265 | 447 | 679 | 302 |

## ipc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 80031 | 1.251s | 79949 | 63984 | 80.0 | 255 | 414 | 679 | 275 |

## inproc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 80020 | 1.269s | 78785 | 63044 | 80.0 | 225 | 319 | 454 | 232 |

## grpc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message Subscribe | 100000 | 80058 | 2.212s | 45218 | 36200 | 80.1 | 6306 | 6335 | 6391 | 6140 |

## 复现

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf
# 仅 message：ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf --features grpc
```
