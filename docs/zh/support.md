[English](../en/support.md) | 中文

# 支持与验证状态

本文描述仓库版本 **2.3.1** 的实现范围与可追溯验证证据（文档整理：2026-09-08）。“配置了 CI 检查”不代表某个提交已通过；使用发布包时还应检查对应 tag 的构建结果和实际附件。

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

## ROS2 桥：目标发行版与证据

目标发行版为 **Humble / Jazzy**，topic、service、action 均有双向桥接实现。两方向需分别配置路由。目标支持不等于所有语言和功能已完成真实发行版验收。

| 语言 | 目标发行版 | Topic | Service / Action | 验证边界 |
| --- | --- | --- | --- | --- |
| Rust / rclrs | Humble、Jazzy | 有 mapper 与路由实现 | 有双向实现 | 常规 CI 使用 Humble 配置的 `ros2-shim` 类型检查，不链接真实 ROS；2026-08-31 的本机记录存在 rust IDL 类型不匹配 |
| Python / rclpy | Humble、Jazzy | 有 mapper 与路由实现 | 有双向实现 | 常规 CI 包含模拟测试和 bus 侧测试；服务及 Action 反馈、结果、超时、取消的真实发行版实测待补 |
| C++ / rclcpp | Humble、Jazzy | 有 mapper 与路由实现 | 有双向实现 | [发布流程](../../.github/workflows/release-cpp.yml) 配置 Linux ROS 包构建；构建与打包不等于逐路由运行验收，服务/Action 真实实测待补 |

历史失败环境与命令见[桥接测试记录](ros2-bridge-perf-report.md)。该记录不代表今天每台机器都会失败，也不能作为当前提交已修复的证据。Rust 除 source 发行版外，还需要与 mapper 匹配的 rust IDL 和 typesupport；仅通过 shim 检查不足以确认可运行。

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

[现有 bus 报告](perf-report.md)是历史单机采样，存在缺测项与 WS 计数异常，不足以用于 ROS2 选型排名。[性能报告阅读与复现要求](performance.md)列出可用结论、待解释数字和重新采样需要的信息。
