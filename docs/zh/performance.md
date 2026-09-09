[English](../en/performance.md) | 中文

# 性能报告阅读与复现要求

仓库性能报告是本机实验记录，不是跨平台承诺，也不足以直接给 Robot Bus 与 ROS2 排名。吞吐、延迟、传输、进程布局和可靠性约束应一起阅读。

## 当前保存结果的边界

以下为 2026-09-08 文档复核时的状态，不是本次重新测量的结果。

| 报告 | 可读取的内容 | 缺口与限制 |
| --- | --- | --- |
| [Robot Bus](perf-report.md) | 部分 TCP/inproc/WS 的 64B 单机结果，以及方法与命令 | 环境仅有版本和逻辑 CPU 数；缺 CPU 型号、OS、提交、测试日期、完整运行配置和重复试验波动；IPC/service/action/federation 为缺失数据 |
| [ROS2](ros2-perf-report.md) | Humble、Fast DDS、SHM/UDP 配置及结果 | 未建立与 bus 报告的同机、同时间、同资源限制证据，不能直接比较倍数 |
| [ROS2 桥](ros2-bridge-perf-report.md) | 2026-08-31 的未执行/构建失败记录与测试方法 | 没有可用吞吐或延迟结果；需要匹配的真实 ROS 环境重新验证 |

`—` 表示报告未提供数据，不表示零性能，也不表示该能力不支持。当前记录没有完整说明各缺测场景的原因，不推测为“已测但未展示”。

## WS 数字需要先核对

保存的 WS Subscribe 行写有 `sent=151447`、`recv=151595`、`delivery=100.1%`。若同一试验内每条消息预期只接收一次，这组计数不能直接解释为成功投递率。是否存在跨试验消息、统计窗口差异或重复计数，需结合原始样本与计数代码调查；本轮只修正文档，没有确认根因或重算数值。

WS Publish 的约 2,659/s 与 WS Subscribe 的约 138,793/s 来自不同方向的场景，不能配对计算同一条端到端链路的损失。报告中的延迟另行采样，不是在所列最大吞吐负载下测出的延迟。

在原始计数与窗口核实前，不使用这些 WS 数字做选型结论。保留历史表格便于追踪，不把异常修饰成正常结果。

## 重跑时记录什么

1. **环境：** 日期、Git commit/tag、CPU 型号与核心数、内存、OS/内核、容器/虚拟机、CPU 配额、编译模式与工具链。ROS 另记发行版、RMW、DDS 和 overlay。
2. **负载：** payload 类型/大小、发布订阅者数量、进程布局、传输、限速与试验时长、HWM/QoS、预热和排空方式、全部环境变量。
3. **原始结果：** 每轮 sent/recv、统计起止、耗时、丢失/重复定义、延迟样本及失败日志。跳过项目注明未运行原因。
4. **重复性：** 同一配置运行多轮并保留每轮数据，报告中位数和波动范围。不要只挑最好一轮。
5. **对比条件：** 在相同机器和资源限制下匹配 payload、进程布局、QoS、统计窗口与运行次数；无法匹配的差异明确列出。

先检查计数自洽，再讨论性能。接收数大于发送数、吞吐窗口不一致、缺少样本等情况应标为待核实；不要据此宣称最大可持续投递率。

## 复现入口

从源码构建的前置步骤见[贡献指南](../../CONTRIBUTING.md)。以下命令会覆盖对应中英文报告，运行前保留历史结果及本轮原始输出。

```bash
just perf
# 仅测 message；该配置不能用于报告 service/action 性能
ROBOT_BUS_PERF_ONLY=message cargo run --release --bin robot_bus_perf
# 仅测邦联（两个 TCP broker，A→B）
ROBOT_BUS_PERF_ONLY=federation cargo run --release --bin robot_bus_perf
# 或
just perf-federation
```

ROS 测试额外依赖 ROS 环境；默认任务使用名为 `ros2` 的容器，不能假定每台开发机都有该容器。具体依赖和可选本机路径见 [ROS bench](../../benches/ros2_perf/README.md) 与 [bridge bench](../../benches/ros2_bridge_perf/README.md)。

```bash
just perf-ros2
just perf-ros2-bridge
```

性能不作为普通文档检查的门槛。发布新的性能结论时，应同时更新原始证据、环境说明和[支持状态](support.md)。
