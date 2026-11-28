# ros2_perf

ROS2（Humble）性能对比小项目：在 Docker容器里测 **pub/sub / service / action**，并固定两种 Fast DDS传输：

| 模式 | 含义 |
|------|------|
| `shm` | Shared Memory **only**（`config/fastdds_shm.xml`，无 UDP） |
| `udp` | UDPv4 only，无 SHM（`config/fastdds_udp.xml`） |

测试方法与结果见 [Robot Bus 与 ROS 2 性能对比](../../docs/zh/performance.md)。

- **延迟**：限速抽样（发一条等收到再发）
- **吞吐（主指标）**：按目标速率限速发送约 **1s**（可用 `ROS2_PERF_GOODPUT_TRIAL_MSGS`改为固定条数），**二分搜索**丢包 ≤ 1% 且发送窗口内 pub/sub均 ≥90% 目标速率的最大可持续速率（max goodput）；默认搜索上限 **500000 Hz**（可用 `ROS2_PERF_GOODPUT_RATE_HI`覆盖）

> 旧实现曾用「条数上限 50000」截断高速率试验（短突发 + settle排空），会虚高 max goodput；现已改为按时长发送，并要求发送窗口内订阅跟上。

## 结构

```
benches/ros2_perf/
  config/fastdds_shm.xml
  config/fastdds_udp.xml
  src/ros2_perf/          # ament包（Echo.srv / Echo.action + C++ bench）
  run.sh                  # 宿主机同步进容器 / 容器内本地跑
```

## 用法

默认用名为 `ros2`的容器（可用 `ROS2_PERF_CONTAINER`覆盖）：

```bash
./benches/ros2_perf/run.sh
```

脚本会同步源码、执行 `colcon build`、运行 SHM 和 UDPv4 测试，再将报告写入 `bench-results/zh/ros2-perf-report.md` 和 `bench-results/en/ros2-perf-report.md`。

已在容器内时：

```bash
cd /path/to/benches/ros2_perf
./run.sh --local
```

冒烟（缩小搜索与试验）：

```bash
ROS2_PERF_MSG_LATENCY_SAMPLES=200 \
ROS2_PERF_GOODPUT_TRIAL_MSGS=1000 \
ROS2_PERF_GOODPUT_RATE_LO=500 \
ROS2_PERF_GOODPUT_RATE_HI=5000 \
ROS2_PERF_SVC_ITERS=1000 ROS2_PERF_ACT_ITERS=1000 \
./benches/ros2_perf/run.sh
```

只跑一种模式：

```bash
./benches/ros2_perf/run.sh --mode shm
```

## 说明

- RMW 固定为 `rmw_fastrtps_cpp`。Humble 在 `ROS_LOCALHOST_ONLY=1` 时会额外添加 UDP 和 SHM，因此此 bench 设置 `ROS_LOCALHOST_ONLY=0`，由 XML 独占选择传输。UDP XML 将接口限定在 `127.0.0.1`。
- 单进程多 Node 和 `MultiThreadedExecutor`。两个 XML 都设 `intraprocess_delivery=OFF`，使消息经指定的 DDS 传输；测量范围为本机。
- Message QoS：`KeepLast(2048)` + **best_effort**。
- `ROS2_PERF_GOODPUT_RATE_STEP` 控制速率档位间距（默认 1 Hz），`ROS2_PERF_GOODPUT_SETTLE_MS` 控制排空时间（默认 100 ms）。`ROS2_PERF_EXECUTOR_THREADS` 默认 2，`ROS2_PERF_RPC_WARMUP` 可统一 service 和 action 的预热次数。
- 标准输出里的 `RAW trial`、`RAW latency` 和 `RAW result` 与 bus 测试使用相同的 JSON 字段。直接运行 `ros2_perf_bench` 时，可通过 `ROS2_PERF_REPORT` 把报告写入本次实验目录。
- 直接运行 `ros2_perf_bench` 时沿用上述环境和 XML。传输复核用固定速率、独立容器网络命名空间以及 `/proc/net/snmp` UDP 计数，证据见[性能说明](../../docs/zh/performance.md)。
- Apple宿主机不直接跑 ROS2；数字来自 Linux容器（Docker Desktop VM），和 macOS上的 `just perf`不在同一 OS，横比时注意环境差异。
- 若报告里 max goodput顶到 `RATE_HI`，把 `ROS2_PERF_GOODPUT_RATE_HI`再调高后重跑。
