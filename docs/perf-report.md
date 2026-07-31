# robot-bus 性能测试报告

由 `just perf`（`cargo run --release --bin robot_bus_perf`）生成。

## 环境

- robot-bus: 0.1.0
- 系统: macOS 26.5.2 (25F84)
- 逻辑 CPU: 10
- CPU: Apple M5
- CPU 核心(物理): 10
- CPU 核心(逻辑): 10
- 内存: 32.0 GiB

## 方法

- 进程内 `RobotBusBroker`，`bind_all_transports = true`（tcp + ipc + inproc + grpc）。
- Console HTTP 关闭；message HWM=2048（仅 bench）；service/action HWM=64。
- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。
- Message **吞吐（主指标）**：在目标速率下限速发送，**二分搜索**丢包率 ≤ 1.0% 的最大可持续速率（max goodput）；每档约 1.0s（可用 `ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS` 覆盖条数）。
- Message **延迟**：另做 5000 次限速抽样（发一条等收到再发），测单程时延。
- Service / action：各 100000 次；延迟为每次 call / send_goal 本地计时。
- ZMQ：共享 `Context` + `Node::tcp` / `ipc` / `inproc`；gRPC：`Node::grpc_at`。
- inproc 与嵌入式 broker 必须共用同一 `Context`（ZeroMQ inproc 是 context-local）。
- 指标为单机本机回环，机器相关，不作为 CI 门槛。

## 横比

message 为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。service/action 为完成速率。gRPC Node **不支持 publish**，发布格为 —。

| 场景 | tcp | ipc | inproc | grpc |
|------|-----|-----|--------|------|
| message 发布 | 47781/s | 38550/s | 46072/s | — |
| message max goodput | 47781/s (100.0% delivered) | 38283/s (99.3% delivered) | 45791/s (99.4% delivered) | 18566/s (99.8% delivered) |
| service call | 5181/s | 6544/s | 10590/s | 529/s |
| action send_goal | 4249/s | 5085/s | 7144/s | 528/s |

## tcp

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 47780 | 47780 | 1.000s | 47781 | 47781 | 100.0 | 370 | 483 | 666 | 364 |
| service call | 100000 | 100000 | 19.301s | 5181 | 5181 | 100.0 | 184 | 256 | 355 | 193 |
| action send_goal | 100000 | 100000 | 23.535s | 4249 | 4249 | 100.0 | 225 | 312 | 420 | 235 |

## ipc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 38550 | 38283 | 1.000s | 38550 | 38283 | 99.3 | 311 | 421 | 527 | 305 |
| service call | 100000 | 100000 | 15.281s | 6544 | 6544 | 100.0 | 149 | 201 | 243 | 153 |
| action send_goal | 100000 | 100000 | 19.664s | 5085 | 5085 | 100.0 | 190 | 260 | 338 | 196 |

## inproc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message pub/sub | 46073 | 45792 | 1.000s | 46072 | 45791 | 99.4 | 194 | 306 | 422 | 202 |
| service call | 100000 | 100000 | 9.443s | 10590 | 10590 | 100.0 | 87 | 141 | 215 | 94 |
| action send_goal | 100000 | 100000 | 13.998s | 7144 | 7144 | 100.0 | 128 | 211 | 290 | 140 |

## grpc

| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|
| message Subscribe | 18602 | 18566 | 1.000s | 18602 | 18566 | 99.8 | 6264 | 6351 | 6484 | 6010 |
| service Call | 100000 | 100000 | 188.955s | 529 | 529 | 100.0 | 722 | 978 | 1180 | 1889 |
| action Run | 100000 | 100000 | 189.540s | 528 | 528 | 100.0 | 780 | 1150 | 1579 | 1895 |

## 复现

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf
# 仅 message：ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf --features grpc
```
