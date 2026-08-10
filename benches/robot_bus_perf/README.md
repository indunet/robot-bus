# robot_bus_perf

robot-bus 本机性能压测：进程内 `RobotBusBroker`，覆盖 **tcp / ipc / inproc / grpc** 的 pub/sub、service、action。

方法与 `benches/ros2_perf` 对齐，报告写入 [`docs/perf-report.md`](../../docs/perf-report.md)：

- **延迟**：限速抽样（发一条等收到再发）
- **吞吐（主指标）**：在目标速率下限速发送，**二分搜索**丢包 ≤ 1% 的最大可持续速率（max goodput）
- **Action**：gRPC 场景名为 `action SendGoal`，对应一元 goal 请求 + server-stream `FEEDBACK` / `RESULT`

## 结构

```
benches/robot_bus_perf/
  main.rs       # cargo bin `robot_bus_perf`
  support.rs    # 报告 / 环境 / ScenarioResult
```

## 用法

```bash
just perf
# 或
cargo run --release --bin robot_bus_perf --features ws
```

仅 message：

```bash
ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf --features ws
```

常用环境变量：

| 变量 | 默认 | 含义 |
|------|------|------|
| `ROBOT_BUS_PERF_MAX_LOSS_PCT` | `1.0` | 丢包阈值（%） |
| `ROBOT_BUS_PERF_GOODPUT_TRIAL_SECS` | `1.0` | 每档试验时长（秒）；默认按时长发送，避免高速率被条数上限截成短突发 |
| `ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS` | （空） | 若设置则改为固定条数（冒烟） |
| `ROBOT_BUS_PERF_GOODPUT_RATE_LO` / `_HI` | `1000` / `2000000` | 二分搜索速率范围（Hz） |
| `ROBOT_BUS_PERF_MSG_LATENCY_SAMPLES` | `5000` | 限速延迟抽样次数 |
| `ROBOT_BUS_PERF_SVC_ITERS` | `10000` | service call 次数 |
| `ROBOT_BUS_PERF_ACT_ITERS` | `5000` | action send_goal 次数 |

## 说明

- Message HWM=2048（仅 bench）；与 trial 长度配合，避免深队列把过载藏掉。
- 指标为单机本机回环，机器相关，不作为 CI 门槛。
- 与 ROS 2 对比见 [`../ros2_perf/`](../ros2_perf/) / [`docs/ros2-perf-report.md`](../../docs/ros2-perf-report.md)；注意 macOS 本机 vs Linux 容器环境差异。
