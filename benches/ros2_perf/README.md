# ros2_perf

ROS 2（Humble）性能对比小项目：在 Docker 容器里测 **pub/sub / service / action**，并固定两种 Fast DDS 传输：

| 模式 | 含义 |
|------|------|
| `shm` | Shared Memory **only**（`config/fastdds_shm.xml`，无 UDP） |
| `udp` | UDPv4 only，无 SHM（`config/fastdds_udp.xml`） |

方法对齐仓库根目录的 `docs/perf-report.md`：

- **延迟**：限速抽样（发一条等收到再发）
- **吞吐（主指标）**：按目标速率限速发送约 **1s**（可用 `ROS2_PERF_GOODPUT_TRIAL_MSGS` 改为固定条数），**二分搜索**丢包 ≤ 1% 且发送窗口内 pub/sub 均 ≥90% 目标速率的最大可持续速率（max goodput）；默认搜索上限 **500000 Hz**（可用 `ROS2_PERF_GOODPUT_RATE_HI` 覆盖）

> 旧实现曾用「条数上限 50000」截断高速率试验（短突发 + settle 排空），会虚高 max goodput；现已改为按时长发送，并要求发送窗口内订阅跟上。

## 结构

```
benches/ros2_perf/
  config/fastdds_shm.xml
  config/fastdds_udp.xml
  src/ros2_perf/          # ament 包（Echo.srv / Echo.action + C++ bench）
  run.sh                  # 宿主机同步进容器 / 容器内本地跑
```

## 用法

默认用名为 `ros2` 的容器（可用 `ROS2_PERF_CONTAINER` 覆盖）：

```bash
./benches/ros2_perf/run.sh
```

会：同步源码 → `colcon build` → 跑 shm + udp → 写回 [`docs/ros2-perf-report.md`](../../docs/ros2-perf-report.md)。

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

- RMW 固定 `rmw_fastrtps_cpp`；`ROS_LOCALHOST_ONLY=1`。
- 单进程多 Node + `MultiThreadedExecutor`，与 robot-bus 本机回环场景同级，不是跨机 DDS。
- Message QoS：`KeepLast(2048)` + **best_effort**。
- 小 payload（64B）+ 同容器本机回环时，**UDP loopback 有时会高于 SHM**：SHM 管理段/同步有固定开销，优势更常体现在大消息或跨进程；且 Docker Desktop 的 `/dev/shm` 常受限。上次结果里 SHM 延迟更低、吞吐更低，符合这一画像。
- Apple 宿主机不直接跑 ROS 2；数字来自 Linux 容器（Docker Desktop VM），和 macOS 上的 `just perf` 不在同一 OS，横比时注意环境差异。
- 若报告里 max goodput 顶到 `RATE_HI`，把 `ROS2_PERF_GOODPUT_RATE_HI` 再调高后重跑。
