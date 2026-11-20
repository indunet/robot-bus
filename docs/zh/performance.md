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

## 预编码 CDR Topic 对照

此前的 Robot Bus Topic 直接收发裸字节，ROS 2 Topic 使用类型化 `UInt8MultiArray`。这组对照让两边都直接传相同布局的 **80 B CDR**，含 16 B 固定前缀（封装和数组元数据）及 64 B 应用数据。ROS 2 在计时前用实际 RMW 生成并校验模板，测量时使用 `GenericPublisher` 与 `GenericSubscription` 收发序列化缓冲区。两边的发布端复用缓冲区，只写入时间戳、试验轮次和序号，订阅端校验前缀并直接读字段。应用层消息编解码在计时外，DDS 和 ZeroMQ 本身的复制、封帧、排队、转发及回调成本包含在各自结果里。 [ROS 2 序列化发布源码](https://github.com/ros2/rclcpp/blob/16.0.21/rclcpp/src/rclcpp/generic_publisher.cpp#L21-L29) · [ROS 2 序列化订阅源码](https://github.com/ros2/rclcpp/blob/16.0.21/rclcpp/src/rclcpp/generic_subscription.cpp#L42-L48)

两边在同一个 Humble 容器顺序运行，CPU 32–39、8 GiB 内存、512 MiB 共享内存。每条路径各一个发布者和订阅者，一个应用层接收执行器线程。Robot Bus 测 TCP、IPC、inproc，ROS 2 测独立配置的 SHM、回环 UDPv4。固定负载为 1,000、10,000、50,000 条/秒，各跑三轮，发送 3 秒，排空 100 ms。单条延迟各测 500 条。负载下对序号能被 16 整除且已送达的消息采样，包含排空窗口内收到的抽样消息。两边均采用尽力而为语义，ZeroMQ HWM 与 DDS KeepLast 各为 2048。

每格为三轮中位数 [最小值–最大值]。接收速率按发送窗口计算，投递率在排空后计算。尾延迟采用已送达样本的最近秩百分位数。

| 目标条数/s | 路径 | 发送/s | 窗口内接收/s | 投递率 (%) | 逐条 p50 (µs) | 负载 p50 (µs) | 负载 p99 (µs) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | tcp | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 145 [141–151] | 121 [121–127] | 192 [146–194] |
| 1,000 | ipc | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 117 [107–122] | 108 [107–109] | 131 [128–143] |
| 1,000 | inproc | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 55 [50–56] | 48 [48–48] | 57 [57–61] |
| 1,000 | shm | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 32 [31–33] | 33 [30–34] | 37 [36–40] |
| 1,000 | udp | 1,000 [1,000–1,000] | 1,000 [1,000–1,000] | 100.00 [100.00–100.00] | 37 [30–43] | 35 [29–37] | 48 [43–49] |
| 10,000 | tcp | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 145 [132–145] | 94 [94–94] | 124 [122–152] |
| 10,000 | ipc | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 108 [100–124] | 93 [93–93] | 111 [107–147] |
| 10,000 | inproc | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 50 [49–52] | 39 [39–39] | 46 [45–50] |
| 10,000 | shm | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 33 [28–33] | 25 [25–25] | 29 [28–29] |
| 10,000 | udp | 10,000 [10,000–10,000] | 10,000 [10,000–10,000] | 100.00 [100.00–100.00] | 36 [35–36] | 29 [29–29] | 34 [33–36] |
| 50,000 | tcp | 50,000 [50,000–50,000] | 49,999 [49,999–49,999] | 100.00 [100.00–100.00] | 125 [116–144] | 70 [69–75] | 179 [161–188] |
| 50,000 | ipc | 50,000 [50,000–50,000] | 49,999 [49,999–49,999] | 100.00 [100.00–100.00] | 123 [108–124] | 52 [48–52] | 117 [115–119] |
| 50,000 | inproc | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 50 [49–55] | 21 [21–21] | 39 [32–39] |
| 50,000 | shm | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 32 [23–33] | 18 [18–18] | 23 [23–23] |
| 50,000 | udp | 50,000 [50,000–50,000] | 50,000 [50,000–50,000] | 100.00 [100.00–100.00] | 37 [36–39] | 21 [21–21] | 27 [27–34] |

在最高测试档 50,000 条/秒，五条路径的窗口内接收速率都接近目标，45 个试验排空后均收齐。同一档下，Robot Bus TCP 和 ROS 2 SHM 的负载 p99 中位数分别为 179 µs 和 23 µs。结果覆盖这些固定速率，最大吞吐量和业务层类型化编解码成本需要独立测量。

每轮日志、环境、执行顺序和样本在本机 `bench-results/20261120-190137-matched-cdr-topic/measurements/`，其中 `execution.tsv` 记录实际 UDP 出包计数，`raw.jsonl` 保存原始计数和延迟样本。

复测入口是[运行脚本](../../benches/topic_cdr_perf/run.sh)和[校验汇总脚本](../../benches/topic_cdr_perf/summarize.py)。在同一 Linux 容器内构建两个 Release 二进制，然后运行以下命令。

```bash
source /opt/ros/humble/setup.bash
cargo build --release --no-default-features --features ws --bin robot_bus_perf
bash benches/ros2_perf/run.sh --local --build-only
source benches/ros2_perf/_ws/install/setup.bash
bash benches/topic_cdr_perf/run.sh \
  target/release/robot_bus_perf \
  benches/ros2_perf/_ws/install/ros2_perf/lib/ros2_perf/ros2_perf_bench \
  bench-results/replay-cdr-topic
python3 benches/topic_cdr_perf/summarize.py bench-results/replay-cdr-topic
```

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
