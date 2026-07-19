# robot-bus 性能测试报告

由 `just perf`（`cargo run --release --bin robot_bus_perf`）生成。

## 环境

- robot-bus: 0.0.6
- 系统: macOS 26.5.2 (25F84)
- 逻辑 CPU: 10
- CPU: Apple M5
- CPU 核心(物理): 10
- CPU 核心(逻辑): 10
- 内存: 32.0 GiB

## 方法

- 进程内 `RobotBusBroker`，`bind_all_transports = true`（tcp + ipc + inproc + grpc）。
- Console HTTP 关闭；message HWM=500000，service/action HWM=1000。
- 各传输迭代次数相同：message 200000（狂发） / service 20000 / action 20000（便于横比）。
- Payload：64 字节 raw（前 8 字节为发送端 Unix 纳秒时间戳，用于延迟）。
- Message：发布端尽快连发 N 条（不等待 ACK），订阅端收满 N 条即结束；message HWM=500000。
- ZMQ：`Node::tcp` / `Node::ipc` / `Node::inproc`；gRPC：`Node::grpc_at`。
- 指标为单机本机回环，机器相关，不作为 CI 门槛。

## 横比

单元格为 **吞吐（次/秒）**。gRPC Node **不支持 publish**，对应格为 —。
ZMQ（tcp / ipc）下 message 发布与订阅测的是同一条 pub→sub 端到端路径；gRPC 仅测 Subscribe。

| 场景 | tcp | ipc | inproc | grpc |
|------|-----|-----|--------|------|
| message 发布 | 42117/s | 42242/s | — | — |
| message 订阅 | 42117/s | 42242/s | — | 35541/s |

## tcp

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message pub/sub | 200000 | 200000 | 4.749s | 42117/s | 2351072 | 4442037 | 4629288 | 2353509 | |
| service call | 20000 | 20000 | 3.138s | 6374/s | 153 | 197 | 277 | 157 | |
| action send_goal | 20000 | 20000 | 3.943s | 5072/s | 191 | 249 | 333 | 197 | |

## ipc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message pub/sub | 200000 | 200000 | 4.735s | 42242/s | 2340224 | 4445283 | 4631179 | 2345357 | |
| service call | 20000 | 20000 | 2.670s | 7492/s | 131 | 169 | 232 | 133 | |
| action send_goal | 20000 | 20000 | 3.384s | 5910/s | 166 | 210 | 274 | 169 | |

## inproc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message pub/sub | — | — | — | — | — | — | — | — | no messages received (ZMQ inproc is context-local; SDK nodes use separate contexts from the broker) |
| service call | — | — | — | — | — | — | — | — | call failed (likely inproc context isolation): service 'perf.inproc.echo' timed out after 0.5s |
| action send_goal | — | — | — | — | — | — | — | — | send_goal failed (likely inproc context isolation): action client timed out after 0.8s |

## grpc

| 场景 | 目标次数 | 完成 | 耗时 | 吞吐 | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) | 备注 |
|------|----------|------|------|------|----------|----------|----------|-----------|------|
| message Subscribe | 200000 | 200000 | 5.627s | 35541/s | 2803150 | 5281165 | 5500786 | 2802459 | |
| service Call | 20000 | 8386 | 4.887s | 1716/s | 537 | 683 | 881 | 583 | |
| action Run | — | — | — | — | — | — | — | — | send_goal failed: gRPC connect failed: transport error |

## 复现

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf
```
