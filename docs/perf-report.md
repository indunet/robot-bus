# robot-bus 性能测试报告

由 `just perf`（`cargo run --release --bin robot_bus_perf`）生成。

## 环境

- robot-bus: 0.0.6
- 构建: release (`cargo run --release`)
- 主机名: macbook-air
- 系统: macOS 26.5.2 (25F84)
- 逻辑 CPU: 10
- CPU: Apple M5
- CPU 核心(物理): 10
- CPU 核心(逻辑): 10
- 内存: 32.0 GiB
- 机型: Mac17,3
- rustc: rustc 1.95.0 (59807616e 2026-04-14)
- 负载说明: 本机回环单进程 broker + SDK；非跨机、非多订阅者压测

## 方法

- 进程内 `RobotBusBroker`，`bind_all_transports = true`（tcp + ipc + inproc + grpc）。
- Console HTTP 关闭；message/service/action HWM=1000。
- 各传输迭代次数相同：message 10000（狂发） / service 5000 / action 1000（便于横比）。
- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。
- Message：发布端尽快连发 N 条（不等待 ACK），订阅端收满 N 条即结束；message HWM=100000。
- ZMQ：`Node::tcp` / `Node::ipc` / `Node::inproc`；gRPC：`Node::grpc_at`。
- 指标为单机本机回环，机器相关，不作为 CI 门槛。

## 横比

单元格为 **吞吐/s · p50(µs)**。gRPC Node **不支持 publish**，对应格为 —。
ZMQ（tcp / ipc）下 message 发布与订阅测的是同一条 pub→sub 端到端路径；gRPC 仅测 Subscribe。

| 场景 | tcp | ipc | inproc | grpc |
|------|-----|-----|--------|------|
| message 发布 | 38004/s · 142261 | 41967/s · 119686 | — | — |
| message 订阅 | 38004/s · 142261 | 41967/s · 119686 | — | 31815/s · 166058 |

## tcp

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message pub/sub | 10000 | 10000 | 0.263s | 38004/s | 142261.0 | 247204.0 | 256525.0 | 140977.2 | |
| service call | 5000 | 5000 | 0.838s | 5967/s | 158.5 | 234.8 | 349.0 | 167.5 | |
| action send_goal | 1000 | 1000 | 0.221s | 4517/s | 200.9 | 354.5 | 470.2 | 221.2 | |

## ipc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message pub/sub | 10000 | 10000 | 0.238s | 41967/s | 119686.0 | 223866.0 | 232920.0 | 120083.1 | |
| service call | 5000 | 5000 | 0.690s | 7242/s | 134.5 | 181.5 | 251.1 | 138.0 | |
| action send_goal | 1000 | 1000 | 0.183s | 5463/s | 171.4 | 268.8 | 382.2 | 182.9 | |

## inproc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message pub/sub | — | — | — | — | — | — | — | — | no messages received (ZMQ inproc is context-local; SDK nodes use separate contexts from the broker) |
| service call | — | — | — | — | — | — | — | — | call failed (likely inproc context isolation): service 'perf.inproc.echo' timed out after 0.5s |
| action send_goal | — | — | — | — | — | — | — | — | send_goal failed (likely inproc context isolation): action client timed out after 0.8s |

## grpc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message Subscribe | 10000 | 10000 | 0.314s | 31815/s | 166058.0 | 291595.0 | 301616.0 | 164494.7 | |
| service Call | 5000 | 5000 | 2.791s | 1791/s | 535.5 | 679.1 | 852.9 | 558.2 | |
| action Run | 1000 | 1000 | 0.598s | 1672/s | 543.8 | 753.2 | 996.0 | 598.0 | |

## 复现

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf
```
