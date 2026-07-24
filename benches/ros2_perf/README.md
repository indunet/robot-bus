# ros2_perf

ROS 2（Humble）性能对比小项目：在 Docker 容器里测 **pub/sub / service / action**，并固定两种 Fast DDS 传输：

| 模式 | 含义 |
|------|------|
| `shm` | Shared Memory only（`config/fastdds_shm.xml`） |
| `udp` | UDPv4 only，无 SHM（`config/fastdds_udp.xml`） |

方法对齐仓库根目录的 `docs/perf-report.md`（64 字节 payload、限速测延迟 + firehose 测吞吐、各 10 万次迭代）。

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

冒烟（减少迭代）：

```bash
ROS2_PERF_MSG_ITERS=1000 ROS2_PERF_SVC_ITERS=1000 ROS2_PERF_ACT_ITERS=1000 ./benches/ros2_perf/run.sh
```

只跑一种模式：

```bash
./benches/ros2_perf/run.sh --mode shm
```

## 说明

- RMW 固定 `rmw_fastrtps_cpp`；`ROS_LOCALHOST_ONLY=1`。
- 单进程多 Node + `MultiThreadedExecutor`，与 robot-bus 本机回环场景同级，不是跨机 DDS。
- Apple 宿主机不直接跑 ROS 2；数字来自 Linux 容器（Docker Desktop VM），和 macOS 上的 `just perf` 不在同一 OS，横比时注意环境差异。
