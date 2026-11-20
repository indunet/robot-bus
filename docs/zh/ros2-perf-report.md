[English](../en/ros2-perf-report.md) | 中文

# ROS 2 Humble 性能测试报告

2026-11-20 修正传输配置后，在与 Robot Bus 相同的主机、容器和资源限制下，单独补测了 Fast DDS SHM 和 UDPv4，各三轮。测试使用 ROS 2 Humble、`rmw_fastrtps_cpp` 6.2.10、Fast DDS 2.6.12、`rclcpp` 16.0.21 和 GCC 11.4 Release 构建。原配置的诊断数据和新样本均已归档。 [性能说明](performance.md)

本页 Topic 测的是类型化 `UInt8MultiArray`，计时包含 ROS 消息编解码。直接收发预编码数据的结果见[预编码 CDR 对照](performance.md#预编码-cdr-topic-对照)。

## 方法

- 单进程多 Node，`MultiThreadedExecutor` 两线程。消息为 64 B 应用负载的 `UInt8MultiArray`，单发布者、单订阅者，QoS 为 `KeepLast(2048)` 和 best effort。
- 两个 Fast DDS XML 都关闭进程内传递，脚本设 `ROS_LOCALHOST_ONLY=0`，由 XML 分别启用 SHM 或仅回环 UDPv4。固定发送 20,000 条的传输检查中，两种配置均收齐。SHM 的 UDP 发包计数为 0，UDPv4 为 20,138。 [SHM 配置](../../benches/ros2_perf/config/fastdds_shm.xml) · [UDPv4 配置](../../benches/ros2_perf/config/fastdds_udp.xml)
- 每档发送 1 秒，排空 100 ms，从 100 到 750,000 Hz 按 100 Hz 步长搜索。通过一档要求排空后丢失率不超过 1%，且发送窗口内收、发速率均达到目标速率的 90%。两种传输所选档位均低于搜索上限。
- 消息单向延迟另采 500 条。Service 和 Action 各预热 8 次后串行执行 32 次，Action 包含一条反馈及最终结果。32 个样本的 RPC p99 对单个慢样本敏感。

## 三轮结果

每格是三轮中位数 [最小值–最大值]。消息速率按发送窗口内同轮唯一接收数计算，投递率按排空后同轮唯一接收数计算。RPC 使用成功完成数和调用耗时。

| 传输 | 场景 | 接收或完成/s | 投递或完成率 (%) | p50 (µs) | p95 (µs) | p99 (µs) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| shm | message pub/sub | 60,020 [59,947–61,699] | 99.5 [99.0–100.0] | 35 [24–35] | 39 [37–39] | 78 [47–91] |
| shm | service call | 28,784 [28,316–31,069] | 100.0 [100.0–100.0] | 31 [30–33] | 43 [41–46] | 50 [44–51] |
| shm | action send_goal | 9,624 [9,535–9,937] | 100.0 [100.0–100.0] | 100 [98–101] | 121 [115–127] | 139 [134–174] |
| udp | message pub/sub | 62,236 [62,162–71,601] | 100.0 [100.0–100.0] | 39 [38–39] | 44 [44–69] | 85 [82–91] |
| udp | service call | 21,130 [17,469–22,764] | 100.0 [100.0–100.0] | 44 [43–57] | 56 [46–58] | 59 [56–60] |
| udp | action send_goal | 8,075 [7,971–8,161] | 100.0 [100.0–100.0] | 122 [122–125] | 134 [129–138] | 144 [139–145] |

这些值覆盖普通的类型化发布、订阅回调和 RPC。Fast DDS 的 SHM transport、Data-sharing 和借用消息的零拷贝路径分别有各自的配置条件。跨产品比较还需要匹配应用路径和负载。 [Fast DDS Data-sharing](https://fast-dds.docs.eprosima.com/en/2.6.x/fastdds/transport/datasharing.html) · [复现与原始数据](performance.md)
