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
- Console HTTP 关闭；message HWM=500000，service/action HWM=1000。
- 各传输迭代次数相同：message / service / action 均为 100000（便于横比）。
- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。
- Message **吞吐**：发布端尽快连发 N 条（不等待 ACK），订阅端收满 N 条即结束。
- Message **延迟**：另做 5000 次限速抽样（发一条等收到再发下一条），测单程时延；不用狂发排队时间冒充延迟。
- Service / action 延迟：每次 call / send_goal 的本地计时。
- ZMQ：共享 `Context` + `Node::tcp` / `ipc` / `inproc`；gRPC：`Node::grpc_at`。
- inproc 与嵌入式 broker 必须共用同一 `Context`（ZeroMQ inproc 是 context-local）。
- 指标为单机本机回环，机器相关，不作为 CI 门槛。

## 横比

单元格为 **吞吐（次/秒）**。gRPC Node **不支持 publish**，对应格为 —。
ZMQ（tcp / ipc）下 message 发布与订阅测的是同一条 pub→sub 端到端路径；gRPC 仅测 Subscribe。

| 场景 | tcp | ipc | inproc | grpc |
|------|-----|-----|--------|------|
| message 发布 | 42437/s | 41520/s | 60435/s | — |
| message 订阅 | 42437/s | 41520/s | 60435/s | 35479/s |

## tcp

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|----------|------|------|------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 100000 | 2.356s | 42437/s | 355 | 569 | 980 | 383 |
| service call | 100000 | 100000 | 18.646s | 5363/s | 179 | 231 | 350 | 186 |
| action send_goal | 100000 | 100000 | 22.352s | 4474/s | 218 | 269 | 380 | 223 |

## ipc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|----------|------|------|------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 100000 | 2.408s | 41520/s | 253 | 483 | 1134 | 313 |
| service call | 100000 | 100000 | 15.988s | 6255/s | 159 | 193 | 228 | 160 |
| action send_goal | 100000 | 100000 | 20.109s | 4973/s | 196 | 247 | 354 | 201 |

## inproc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|----------|------|------|------|----------|----------|----------|-----------|
| message pub/sub | 100000 | 100000 | 1.655s | 60435/s | 161 | 295 | 605 | 191 |
| service call | 100000 | 100000 | 8.573s | 11665/s | 84 | 112 | 146 | 86 |
| action send_goal | 100000 | 100000 | 11.740s | 8518/s | 115 | 149 | 193 | 117 |

## grpc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |
|------|----------|------|------|------|----------|----------|----------|-----------|
| message Subscribe | 100000 | 100000 | 2.819s | 35479/s | 6300 | 6429 | 9038 | 6186 |
| service Call | 100000 | 100000 | 186.950s | 535/s | 475 | 674 | 1048 | 1869 |
| action Run | 100000 | 100000 | 186.309s | 537/s | 473 | 732 | 1012 | 1863 |

## 复现

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf
```
