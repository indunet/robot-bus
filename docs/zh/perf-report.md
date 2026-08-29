[English](../en/perf-report.md) | 中文

# robot-bus性能测试报告

由 `just perf`（`cargo run --release --bin robot_bus_perf`）生成。

## 环境

- robot-bus: 2.1.0
- 逻辑 CPU: 10

## 方法

- 进程内 `RobotBusBroker`，`bind_all_transports = true`（tcp + ipc + inproc + ws）。
- Console HTTP关闭；message HWM=2048（仅 bench）；service/action HWM=64。
- Payload：64 字节 raw（前 8 字节为发送端 Unix纳秒时间戳，用于延迟）。
- Message **吞吐（主指标）**：按目标速率限速发送约 1.0s（可用 `ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS`改为固定条数），**二分搜索**丢包率 ≤ 1.0% 且发送窗口内 pub/sub均 ≥90% 目标速率的最大可持续速率（max goodput）。
- Message **延迟**：另做 5000 次限速抽样（发一条等收到再发），测单程时延。
- Service / action：各 10000 / 5000 次（`ROBOT_BUS_PERF_SVC_ITERS` / `ROBOT_BUS_PERF_ACT_ITERS`）；延迟为每次 call / send_goal本地计时。
- ZMQ：共享 `Context` + `Node::tcp` / `ipc` / `inproc`；WebSocket RPC：`Node::ws_at`。
- inproc与嵌入式 broker必须共用同一 `Context`（ZeroMQ inproc是 context-local）。
- 指标为单机本机回环，机器相关，不作为 CI门槛。

## 横比

message为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。service/action为完成速率。

| 场景 | tcp | ipc | inproc | ws |
|------|-----|-----|--------|------|
| message发布 | 401123/s | — | 803213/s | 2659/s |
| message max goodput | 397953/s (99.2% delivered) | — | 796263/s (99.1% delivered) | 138793/s (100.1% delivered) |
| service call | — | — | — | — |
| action send_goal | — | — | — | — |

## tcp

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 401124 | 397973 | 1.000s | 401123 | 397953 | 99.2 | 272 | 346 | 422 | 229 |

## ipc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | — | — | — | — | — | — | — | — | — | — |

## inproc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 803213 | 796272 | 1.000s | 803213 | 796263 | 99.1 | 135 | 225 | 430 | 137 |

## ws

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message Publish | 2660 | 2640 | 1.000s | 2659 | 2639 | 99.2 | 564 | 702 | 809 | 474 |
| message Subscribe | 151447 | 151595 | 1.000s | 151446 | 138793 | 100.1 | 6764 | 7359 | 7628 | 6717 |

## 复现

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf
# 仅 message：ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf --features ws
```
