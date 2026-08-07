# robot-bus 性能测试报告

由 `just perf`（`cargo run --release --bin robot_bus_perf`）生成。

## 环境

- robot-bus: 0.1.6
- 逻辑 CPU: 24

## 方法

- 进程内 `RobotBusBroker`，`bind_all_transports = true`（tcp + ipc + inproc + grpc）。
- Console HTTP 关闭；message HWM=2048（仅 bench）；service/action HWM=64。
- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。
- Message **吞吐（主指标）**：按目标速率限速发送约 1.0s（可用 `ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS` 改为固定条数），**二分搜索**丢包率 ≤ 1.0% 且发送窗口内 pub/sub 均 ≥90% 目标速率的最大可持续速率（max goodput）。
- Message **延迟**：另做 5000 次限速抽样（发一条等收到再发），测单程时延。
- Service / action：各 10000 / 5000 次（`ROBOT_BUS_PERF_SVC_ITERS` / `ROBOT_BUS_PERF_ACT_ITERS`）；延迟为每次 call / send_goal 本地计时。
- ZMQ：共享 `Context` + `Node::tcp` / `ipc` / `inproc`；gRPC：`Node::grpc_at`。
- inproc 与嵌入式 broker 必须共用同一 `Context`（ZeroMQ inproc 是 context-local）。
- 指标为单机本机回环，机器相关，不作为 CI 门槛。

## 横比

message 为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。service/action 为完成速率。gRPC Node **不支持 publish**，发布格为 —。

| 场景 | tcp | ipc | inproc | grpc |
|------|-----|-----|--------|------|
| message 发布 | 500000/s | 500000/s | 500000/s | — |
| message max goodput | 499981/s (100.0% delivered) | 499815/s (100.0% delivered) | 499992/s (100.0% delivered) | 51950/s (99.5% delivered) |
| service call | — | — | — | — |
| action send_goal | — | — | — | — |

## tcp

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 500000 | 500000 | 1.000s | 500000 | 499981 | 100.0 | 133 | 227 | 295 | 142 |

## ipc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 500000 | 500000 | 1.000s | 500000 | 499815 | 100.0 | 111 | 182 | 223 | 115 |

## inproc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 500000 | 500000 | 1.000s | 500000 | 499992 | 100.0 | 65 | 111 | 144 | 68 |

## grpc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message Subscribe | 57664 | 57382 | 1.000s | 57663 | 51950 | 99.5 | 5222 | 5539 | 5569 | 5560 |

## 复现

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf
# 仅 message：ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf --features grpc
```
