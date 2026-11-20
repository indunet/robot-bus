[English](../en/performance.md) | 中文

# 性能报告阅读与复现要求

## 2026-11-20 同机复测

Robot Bus 和 ROS 2 Humble 最初在同一个 Docker 容器中顺序运行，各测三轮。9 次进程运行均成功退出。初始归档包含 306 个吞吐试验、24 组单向延迟样本和 42 组 RPC 样本。后续传输复核发现，初始 ROS 跑法没有隔离 SHM 和 UDPv4，修正配置后在同一容器补测了 ROS 2。

| 报告 | 本次覆盖范围 |
| --- | --- |
| [Robot Bus](perf-report.md) | TCP、IPC、inproc、WS 和双 broker 邦联的消息、Service、Action |
| [ROS 2 Humble](ros2-perf-report.md) | 修正配置后 Fast DDS SHM 和 UDPv4 的消息、Service、Action |
| [ROS 2 桥](ros2-bridge-perf-report.md) | Humble 和 Jazzy 构建与路由验收，桥接吞吐和延迟待采样 |

AMD EPYC 9554 主机（256 逻辑 CPU、755 GiB 内存），Linux 5.15.0-203-generic，容器为 `ros:humble-ros-base`（Ubuntu 22.04.5，x86_64），Docker 29.1.3。容器绑定逻辑 CPU 32–39（8 个物理核心），内存上限 8 GiB，`/dev/shm` 512 MiB。Robot Bus 使用 Rust 1.100.0、ZeroMQ 4.3.4，ROS 2 使用 GCC 11.4、Fast DDS 2.6.12，两者均为 Release 构建。采样源码以 `b9b414e35f0274bdcef19843562ca8ca770d5685` 为基线，应用了归档中的 benchmark 修订。

这些数据适用于各自的本机 API 路径。Robot Bus 传输原始字节并经过 broker，ROS 2 使用 `UInt8MultiArray` 类型、DDS 和两线程执行器。ZeroMQ HWM 与 DDS KeepLast 都设为 2048，但作用于不同的队列。修正后的 ROS 2 数据与初始 Robot Bus 数据分属两次测量。跨产品性能比较仍需统一应用负载、进程和线程布局，并使用共同的固定速率复测。

## ROS 2 传输复核

初始测试把发布者和订阅者放在同一进程，Fast DDS 2.6 默认启用进程内传递，可以绕过传输。在原二进制和容器里，以 20,000 msg/s 两次复跑，原 UDP 配置每轮只发出 21 个 UDP 数据报，关闭进程内传递后则超过 80,000 个。关闭进程内传递后，原 SHM 配置也仍发出约 20,000 个 UDP 数据报，因为 Humble 的 `ROS_LOCALHOST_ONLY=1` 会向 XML 配置额外加入 UDP 和 SHM。原 SHM 和 UDP 两行因此只能作为当时的同进程配置记录。 [Fast DDS 进程内传递](https://fast-dds.docs.eprosima.com/en/2.6.x/fastdds/transport/intraprocess.html) · [Humble RMW 源码](https://github.com/ros2/rmw_fastrtps/blob/6.2.10/rmw_fastrtps_shared_cpp/src/participant.cpp#L145-L162)

修正后的两个 XML 都关闭进程内传递，运行脚本设 `ROS_LOCALHOST_ONLY=0`，UDP XML 仅绑定回环。固定发送约 20,000 条时，两种配置均收齐，单独运行 SHM 时 UDP 发包计数为 0，运行 UDP 时约为 20,000。[ROS 2 报告](ros2-perf-report.md)采用修正后的数据。

在旧配置的固定 80,000 msg/s 诊断中，只把执行器由两线程改为一线程，发送窗口内接收量从约 51,000 条升至 80,000 条，说明旧数据也对执行器调度敏感。吞吐搜索结果表示该次搜索所遇到的最高通过档位。各档需独立重复才能判断速率附近的波动。

## 计数口径和 WS 验证

消息速率用发送窗口内的同轮唯一接收数除以实际发送时长。投递率用排空后的同轮唯一接收数除以发送数。消息延迟另做逐条采样，报表取每轮分位数的中位数和最小–最大范围。RPC 速率按成功完成数除以该轮测量时长。

历史 WS Subscribe 记录为 `sent=151447`、`recv=151595`、`delivery=100.1%`。旧计数会把试验切换时仍在队列里的消息计入新一轮，历史日志无法逐条归属多出的 148 个事件。新版本用轮次和序号区分同轮唯一接收、跨轮消息和重复。复测的 77 个 WS 吞吐试验均满足 `recv_send <= recv_final <= sent`。WS Publish 与 WS Subscribe 各自测量独立方向。

## 归档与复现

初始证据保存在本机 `bench-results/20261120-134156-same-host/`。其中 `run.sh` 是实际运行脚本，`execution.tsv` 记录轮次和退出码，`raw.jsonl` 包含每档计数、窗口时间和延迟样本，`round-*.log` 含完整输出。同目录保留环境、DDS 配置、源码补丁、Cargo 依赖锁定、二进制哈希、`SHA256SUMS` 和一次超时的 Bus 冒烟。传输复核和修正后的 ROS 2 样本分别位于 `bench-results/20261120-183629-ros2-executor-probe/transport/` 与 `isolated/`，各有 XML、环境、UDP 计数和完整日志。

从源码构建的前置步骤见[贡献指南](../../CONTRIBUTING.md)。按场景运行的命令和依赖见 [Robot Bus bench](../../benches/robot_bus_perf/README.md)、[ROS 2 bench](../../benches/ros2_perf/README.md)和[桥接 bench](../../benches/ros2_bridge_perf/README.md)。采样时用 `ROBOT_BUS_PERF_SKIP_REPORT=1`，直接运行 `ros2_perf_bench` 时用 `ROS2_PERF_REPORT` 指定实验目录，保留每轮 stdout 和 stderr。`just perf`、`just perf-ros2` 和 `just perf-ros2-bridge` 默认会写回对应报告。

每次正式复测记录以下信息。

1. 日期、提交和源码差异、CPU 型号和核心数、内存、OS/内核、容器资源限制、Release 构建工具链、ROS 发行版和 RMW/DDS 版本。
2. 消息类型/大小、收发端数量、进程布局、传输、HWM/QoS、试验时长与排空方式和全部环境覆盖项。
3. 每轮原始计数、统计窗口、延迟样本、失败或跳过记录，以及多轮中位数和波动范围。
