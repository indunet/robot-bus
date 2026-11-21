[English](../en/support.md) | 中文

# 支持与验证状态

本文记录仓库 **2.3.2** 开发版本的实现范围与验证证据。具体提交和发布包的验证结果请查看对应的 CI 运行及附件。

## 核心 SDK

未启用 ROS2 桥时，核心 SDK 不需要 ROS。下表依据[常规 CI 配置](../../.github/workflows/ci.yml)，不是所有系统、架构和语言版本的认证清单。

| 客户端 | 能力范围 | 仓库中的检查 | 尚不能据此保证 |
| --- | --- | --- | --- |
| Rust | 原生 topic/service/action、WS 客户端、broker | 原生及 feature 组合测试，示例构建，跨语言互通 | 所有目标平台上的运行表现 |
| Python | 原生 topic/service/action、WS 客户端、broker | Python 3.12 下的 typed、原生和互通测试；README/API 上手代码运行检查 | 所有声明支持的 Python 版本均已逐一运行 |
| TypeScript / Node.js | 原生 SDK、WS 客户端、broker | npm 测试与原生互通 | 所有 Node.js 版本与平台组合 |
| 浏览器 | WS 发布、订阅、service/action 客户端 | WS 协议及客户端测试 | 完整浏览器兼容矩阵；不能作为 service/action 服务端 |
| C++ / Java | 原生 SDK、WS 客户端、broker | 原生接口测试与互通 | 每个发布包都已在用户目标环境运行 |
| Android | 独立 Kotlin SDK，minSdk 24 | JNI 构建、AAR 构建、宿主 JVM 单测 | 真机、模拟器、后台生命周期与断网恢复已全面验证 |

各语言接口差异以对应 [API 指南](../../README-zh.md#16-文档) 为准。WS Node 是客户端模式，不提供 service/action 服务端注册。

[发布工作流](../../.github/workflows/release-cpp.yml) 的 `sdk-smoke` 会下载 Linux amd64 和 Windows x64 核心 SDK 压缩包，并在未安装 protobuf 和 Rust 的 runner 上用 FetchContent 编译一个消费者（protobuf 往返和一个进程内 node）。同一工作流还会上传 arm64、macOS 和 ROS 2 压缩包；这次冒烟覆盖上面两个核心包。

## 可靠性回归测试

Linux Rust CI 使用真实 TCP、WebSocket、broker 和原生 worker 执行以下检查。

| 场景 | 检查项 |
| --- | --- |
| 强制结束 broker 进程并在原端口重启 | 已有 WS 订阅恢复，仍在运行的原生 Service 和 Action worker 可以接收新请求。在途 Service 请求断开后失败，重连时不会重放。 |
| 静默中断 WS 的 TCP 链路 | broker 保持运行。客户端通过心跳发现断网，恢复网络后原订阅和新 Service 请求可用。 |
| 慢订阅者积压 | 暂停回调期间送达 128 条消息，深度为 3 和 1 的两个过滤器分别保留最新 3 条和 1 条。另一客户端收到全部序号，暂停回调的连接也能处理 RPC。 |
| 小长跑 | 预热 5 秒后测量 6 个 10 秒窗口，持续发送 Topic 和 Service 流量并反复创建和注销订阅，记录 RSS、打开的 Linux 文件描述符和两类流量的 p95 延迟。 |

长跑在同一进程内测量 broker 和客户端。预热后，峰值 RSS 增量上限为 32 MiB，文件描述符增量上限为 6。每类延迟用前两个窗口的较高 p95 作基线，最后两个窗口的较高 p95 必须低于或等于基线的 3 倍与基线加 20 ms 中的较大值。可用 `ROBOT_BUS_SOAK_WINDOW_SECONDS` 加长测量窗口。

```sh
export TOKIO_WORKER_THREADS=2
cargo test --no-default-features --features ws,console-api --test ws_reliability -- --nocapture --test-threads=1
cargo test --no-default-features --features ws,console-api --test ws_reliability -- --ignored --nocapture --test-threads=1
```

CI 在 `rust-reliability-evidence` 附件保存故障测试和长跑日志，长跑每个窗口留一条 JSON 记录。见[测试套件](../../tests/ws_reliability.rs)和 [CI 工作流](../../.github/workflows/ci.yml)。

## ROS2 桥：目标发行版与证据

目标发行版为 **Humble / Jazzy**，topic、service、action 均有双向桥接实现。两方向需分别配置路由。目标支持不等于所有语言和功能已完成真实发行版验收。

| 语言 | 目标发行版 | Topic | Service / Action | 验证边界 |
| --- | --- | --- | --- | --- |
| Rust / rclrs | Humble、Jazzy | 有 mapper 与路由实现 | 有双向实现 | 2026-11-20 在 Humble 和 Jazzy 上编译并链接通过，验证了按需话题转发、双向 Trigger 服务和双向 Fibonacci Action |
| Python / rclpy | Humble、Jazzy | 有 mapper 与路由实现 | 有双向实现 | [浏览器验收](ros2-web.md)在 Humble 和 Jazzy 上运行双向话题和浏览器发起的服务、Action 测试，结果见各发行版运行附件 |
| C++ / rclcpp | Humble、Jazzy | 有 mapper 与路由实现 | 有双向实现 | [发布流程](../../.github/workflows/release-cpp.yml) 配置 Linux ROS 包构建；构建与打包不等于逐路由运行验收，服务/Action 真实实测待补 |

Rust 构建复测使用 Ubuntu 22.04.5 和 24.04.5（x86_64，`ros:humble-ros-base` 和 `ros:jazzy-ros-base`）、Rust 1.100.0、`rclrs` 0.8.0、`ros-env` 0.3.0、`rosidl_runtime_rs` 0.7.0 和 Protobuf 编译器 35.1。使用各发行版系统包提供的 Rust 绑定，编译并链接了桥库、`ros2_bridge_perf` 和两个桥接示例。依赖安装及复测命令见[桥接指南](ros2-bridge.md)。2026-08-31 的字段类型失败见[历史性能记录](ros2-bridge-perf-report.md)。

在两个发行版上都使用 Fast DDS 和本机 ROS 通信，通过了 `ros2_bridge` 集成测试中的 `ros2_bridge_lazy`、`ros2_bridge_service_smoke` 和 `ros2_bridge_diagnostics` 三组，共 16 项。两个发行版也各通过了 32 项 Rust 桥单元测试。性能工具的临时副本使用显式 broker 端点和可靠 ROS QoS，完成了双向 Fibonacci 目标。集成测试命令见[桥接指南](ros2-bridge.md)。

在将桥用于应用前，按[桥接示例](../../examples/ros2_bridge/README.md)验证自己的消息类型与方向，并记录：提交/tag、系统/架构、ROS 发行版、RMW、语言、依赖版本、运行命令及结果。Service 验证成功和超时；Action 另验拒绝、反馈、结果、取消及异常。新增验证结论时附可复现证据，不只改“支持”标签。

## 协议兼容与升级

| 变化 | 兼容边界 | 升级方式 |
| --- | --- | --- |
| WS V2 → V3 | V2 客户端不能连接 V3 `/ws-rpc` | 协调升级 broker 与 SDK；不要混用 V2/V3 |
| WS KeepLast | 新客户端使用 opcode 5；旧 V3 broker 可能不识别 | 先升级 broker，再升级客户端；详见 [QoS](rust-api.md#高水位hwm与-qos) |
| 桥接 Action 错误分类 | 旧 SDK 无法分类新增的拒绝/中止/超时错误 | 同步升级桥、客户端与 WS 服务端；详见[桥接诊断](ros2-bridge.md#action-结束状态与调用诊断) |
| 自定义 Protobuf | 业务方维护消息契约 | 保留字段编号，不复用已删除字段；部署前做跨版本编解码与业务语义验证 |

Node 编程模型的稳定性不表示底层线协议和所有错误分类永远不变。重连不会恢复 broker 重启前的 service/action 调用；重试必须使用新的请求/目标 ID，并由业务处理重复执行。

## 性能证据

2026-11-20，[Robot Bus](perf-report.md) 和 [ROS 2 Humble](ros2-perf-report.md) 报告各保留了三轮。传输复核隔离 SHM 与 UDPv4 后，ROS 2 在同一主机和资源限制下单独补测。WS 接收计数通过轮次与序号校验。环境、统计口径和证据见[性能说明](performance.md)。[桥接性能](ros2-bridge-perf-report.md)待单独采样。
