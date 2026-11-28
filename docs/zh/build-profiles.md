# 按需构建

Rust 包 `robot-bus` 同时包含客户端 SDK 和 broker。默认 feature（`full`）包含 broker、WebSocket、内嵌控制台和 tank 演示，所以大多数程序只需写 `robot-bus = "3.3.0"`。下表说明如何选择更精简的构建。

WebSocket 是核心通信能力。`ws` feature 仅用于编译时选择通信方式；控制台和演示属于独立组件。

Cargo feature 是叠加关系。要构建精简版本，必须加 `--no-default-features`；只加 feature 不会移除默认功能。

| 构建版本 | Cargo 参数 | 包含内容 |
| --- | --- | --- |
| 原生 ZMQ SDK | `--no-default-features` | 只有客户端 SDK（`Node`、executor、客户端），不含 broker |
| Broker | `--no-default-features --features broker` | SDK + `RobotBusBroker`、联邦、`robot_bus_broker` / `rbus` 可执行文件 |
| WebSocket 通信 | `--no-default-features --features ws` | Broker + WS 客户端/服务端、发现和订阅队列 API（`ws` 自动启用 `broker`） |
| 监控 API | `--no-default-features --features ws,console-api` | WebSocket 通信 + 监控 REST API 和总线控制服务 |
| 内嵌控制台 | `--no-default-features --features ws,console` | 监控 + 内嵌网页，不含坦克仿真 |
| 完整版（默认） | 不加参数，或 `--features full` | WebSocket 通信 + 控制台 + 坦克演示 |

`console-api` 也可独立使用，不启用 `ws` 时通过 `ConsoleBrokerConfig.listen` 提供 HTTP 监控。`console` 自动启用 `console-api`，并嵌入 `crates/robot-bus/assets/console`；`demo-tank` 独立控制 Rust 坦克仿真代码。不带 `demo-tank` 时，界面隐藏坦克入口，申请会话返回 403，即使运行配置显式打开坦克也不会启动。启用 `console` 时，共用网页资源仍包含坦克视图代码。

从源码构建，先运行 `just gen-rust` 生成 protobuf 文件。只有带 `console` 的构建需要 `just console`（Node/npm/pnpm 及 TypeScript 代码生成）。原生 ZMQ、WS 通信和监控 API 无需网页资源或前端构建工具。Cargo 使用原生 C/C++ 工具链编译并静态链接随构建提供的 libzmq 源码。已发布的 Rust 包包含生成文件。

快捷命令：`just build-sdk`、`just build-ws`、`just build-monitoring`、`just build-full`。均为 release 构建；SDK 命令构建库，其他命令构建 `robot_bus_broker`。

精简 Rust 依赖示例：

```toml
# 只用客户端 SDK
robot-bus = { version = "3.3.0", default-features = false }
# SDK + broker + WebSocket，不含控制台和 tank 演示
# robot-bus = { version = "3.3.0", default-features = false, features = ["ws"] }
```

默认 Cargo 构建和官方 Python、Node、C++/Java/Android 发行包继续保留完整功能。原来显式使用 `--no-default-features --features ws,console` 的构建若需要坦克，应加上 `demo-tank`。运行参数 `--no-console`、`--no-tank` 用于关闭行为，编译 feature 用于移除组件。
