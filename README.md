# *Robot Bus*

[![CI](https://github.com/indunet/robot-bus/actions/workflows/ci.yml/badge.svg)](https://github.com/indunet/robot-bus/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/robot-bus.svg?color=f74c00)](https://crates.io/crates/robot-bus)
[![PyPI](https://img.shields.io/pypi/v/robot-bus.svg?color=3775a9)](https://pypi.org/project/robot-bus/)
[![License](https://img.shields.io/badge/License-Apache_2.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

轻量级、免环境配置的 ROS 2 风格通信库：基于 ZeroMQ，提供 topic / service / action，以及 `Executor` + `Node` + `spin` 回调模型。

不依赖 ROS 发行版、不需要 `source setup.bash`、不搭 workspace。一个 broker 进程 + SDK（Rust / Python）即可。

**设计原则**：API 会尽量贴近 ROS 2 的用法与命名（如 `Node`、`SingleThreadedExecutor` / `MultiThreadedExecutor`、`add_node`、`create_publisher` / `create_subscription`、`spin`），降低从 ROS 2 迁过来的心智负担；底层用 ZeroMQ 实现，不绑定某一 ROS 发行版。

> **预发布说明**：当前仍处于预发布阶段。接下来 API 可能会有较多变更，运行稳定性也尚不完善，请谨慎用于生产环境。

更多 API 示例见 [`docs/`](docs/)。

| 模块 | 职责 |
|------|------|
| `broker::` | 路由进程（message / service / action） |
| 顶层 API | Publisher / Subscriber / Client / Worker |
| `runtime::Executor` | 底层 poll loop（一般用下面两个包装） |
| `runtime::SingleThreadedExecutor` / `MultiThreadedExecutor` | 显式执行器（多节点 / 并行）；单节点可直接 `Node::spin` |
| `runtime::Node` / `TopicPublisher` / `CallbackGroup` | 节点、publisher、callback group（互斥 / 可重入） |
| `grpc::`（默认 feature） | gRPC / gRPC-Web 网关（随 broker 一起启动） |
| [`proto/`](proto/) | ROS 风格 Protobuf：`proto/<pkg>/{msg\|srv\|grpc}/v1/` → Rust/Python `robot_bus.<pkg>…` |
| [`console/`](console/) | Web 监控控制台（可选；独立前端，不随 crates.io / PyPI 发布） |

## 架构

```
业务代码 (Rust / Python)
  └── robot-bus SDK
              │
              │ ZMQ (tcp / ipc / inproc)
              ▼
robot_bus_broker 进程
```

## 快速开始

### 1. 启动 broker

Rust：

```bash
cargo run --bin robot_bus_broker
```

Python（`pip install robot-bus` 后自带可执行入口）：

```bash
robot-bus-broker
```

或在代码里进程内启动：

```python
import robot_bus

with robot_bus.RobotBusBroker.start() as broker:
    # ... 业务代码 ...
    pass
# 离开 with 自动 stop

# 或阻塞运行（等同于命令行，Ctrl+C 退出）
# robot_bus.run_broker()
```

### Python

```bash
pip install robot-bus
```

本地开发（需 [maturin](https://www.maturin.rs/)）：

```bash
maturin develop --features extension-module,grpc
```

（`grpc` 为默认 feature；显式写出可避免 `default-features = false` 的构建漏掉网关。）

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, imu: Imu):
    print(topic, imu.linear_acceleration)

node = robot_bus.Node("pilot")

imu_pub = node.create_publisher("/robot1/imu", Imu)
node.create_subscription("/robot1/imu", on_imu, msg_type=Imu)
imu_pub.publish(Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8)))
# node.spin()  # 阻塞；另线程调用 node.shutdown() / shutdown_handle().shutdown()
```

（不传消息类型时仍为 raw bytes。多节点共享或需多线程 handler 时再用 `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node`。）

### 2. Rust（Node + spin）

在 `Cargo.toml` 中添加依赖：

```toml
robot-bus = { path = "../robot-bus" }
# 或 crates.io：robot-bus = "0.0.4"
```

语义接近 ROS 2：`Node::new` → typed `create_publisher` / `create_subscription` → `node.spin()`（自动挂 `SingleThreadedExecutor`）：

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::Node;

let mut node = Node::new("pilot");

let imu_pub = node.create_publisher::<Imu>("/robot1/imu")?;
node.create_subscription::<Imu, _>(
    "/robot1/imu",
    |topic, imu| {
        println!("{topic}: {:?}", imu.linear_acceleration);
    },
    None,
)?;

let imu = Imu {
    linear_acceleration: Some(Vector3 { x: 0.0, y: 0.0, z: 9.8 }),
    ..Default::default()
};
imu_pub.publish(&imu)?;

node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| {
        // 控制周期 / 心跳
    }),
    None,
)?;

let handle = node.shutdown_handle()?;
std::thread::spawn(move || { /* ... */ handle.shutdown(); });
node.spin()?;
```

- 单节点默认：直接 `node.spin()`（内部 `SingleThreadedExecutor`）
- `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node`：多节点共享或并行 handler
- Callback group：`MutuallyExclusive` / `Reentrant`（`create_callback_group`；默认互斥组）
- Service / action：typed `create_service` / `create_client`、`create_action_server` / `create_action_client`（与 topic 一样挂在 Node；另有 `*_raw`）
- Timer：`create_timer`（同样挂在 Node，由 `spin` 驱动）
- Raw bytes：`create_publisher_raw` / `create_subscription_raw`
- 底层 escape hatch：`Executor`（高级用法）
发送 / 接收水位（ZMQ HWM，不是完整 QoS）可在创建时或运行中设置：

```rust
use robot_bus::{Publisher, HighWaterMark};

let pub_ = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
pub_.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

默认：message `STREAM(2/2)`、service `RPC(4/4)`、action `ACTION(8/8)`。Broker 侧用 `--snd-hwm` / `--rcv-hwm`。

## 二进制

| 二进制 | 说明 |
|------|------|
| `robot_bus_broker` | 一次启动三条总线 + gRPC / gRPC-Web 网关 |

## Web 控制台（`console/`）

可选的监控前端：查看 broker 状态、topic 流量与事件日志。当前为 UI 原型（mock 数据），需本机安装 [pnpm](https://pnpm.io/)。

```bash
cd console
pnpm install
pnpm dev
# 浏览器打开 http://localhost:3000
```

不随 `robot-bus` 的 crates.io / PyPI 包发布；与 broker 的真实监控 API 对接仍在规划中。

## gRPC / gRPC-Web 网关

随 `robot_bus_broker` / `RobotBusBroker::start` 一起启动；标准 gRPC 与 gRPC-Web **同端口**（默认 `0.0.0.0:15770`）。

| RPC | 语义 |
|-----|------|
| `MessageGateway.Subscribe` | 按 topic 前缀订阅，服务端流式返回二进制 payload |
| `ServiceGateway.Call` | 一元：`service_name` + request bytes → response bytes |
| `ActionGateway.Run` | 双向流：客户端发 GOAL / CANCEL，服务端推 `ActionEvent`（`kind` 区分 FEEDBACK / RESULT） |

```bash
cargo run --bin robot_bus_broker
# 配置：cargo run --bin robot_bus_broker -- --help
# gRPC: http://0.0.0.0:15770
```

进程内：

```rust
use robot_bus::{GrpcBrokerConfig, RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig {
    grpc: GrpcBrokerConfig {
        listen: "0.0.0.0:15770".parse()?,
        ..Default::default()
    },
    ..RobotBusConfig::default()
})?;
let grpc = format!("http://{}", broker.grpc_listen());
```

Proto（包名 `robot_bus.grpc.v1`，与 ROS `*.msg.v1` / `*.srv.v1` 区分）：

- [`message_gateway.proto`](proto/robot_bus/grpc/v1/message_gateway.proto)
- [`service_gateway.proto`](proto/robot_bus/grpc/v1/service_gateway.proto)
- [`action_gateway.proto`](proto/robot_bus/grpc/v1/action_gateway.proto)

## 测试

```bash
cargo test
PYTHONPATH=python python3 tests/python/test_msgs_roundtrip.py
PYTHONPATH=python python3 tests/python/test_typed_api.py
```

## Protobuf 消息

[`proto/`](proto/) 按 ROS 包布局：`proto/<pkg>/{msg|srv|grpc}/v1/*.proto`。

| 语言 | 路径 | 说明 |
|------|------|------|
| Rust | `robot_bus::<pkg>::{msg\|srv}::v1` | `build.rs` + prost；挂在 crate 命名空间下 |
| Python | `robot_bus.<pkg>.{msg\|srv}.v1` | 随 wheel 打包；`scripts/generate_python_msgs.py` 生成 |

- 传输层 body 仍是 opaque bytes（含 gRPC 网关）；Rust Node SDK 在 create 时绑定类型并自动 encode/decode（`create_publisher::<M>` 等），也可用 `*_raw`；Python 传入 protobuf 类即可 typed（纯 Python 薄封装），省略类型则为 raw bytes
- **srv** 是一对 `*Request` / `*Response` message，不是 gRPC
- **grpc**（`robot_bus`）是网关 RPC 契约，随 broker 启动（默认 feature `grpc`）
- 消息在 `robot_bus` 命名空间下，**不占用** ROS 顶层 `sensor_msgs` 包名；编码是 protobuf，与 ROS CDR 不互通
- 改 proto 后跑：`python3 scripts/generate_python_msgs.py`（建议 protoc 28.x，与 CI 一致）

已覆盖：`builtin_interfaces`、`std_msgs`、`std_srvs`、`geometry_msgs`、`sensor_msgs`、`nav_msgs`、`tf2_msgs`、`trajectory_msgs`、`diagnostic_msgs`、`unique_identifier_msgs`、`shape_msgs`、`visualization_msgs`、`control_msgs`、`nav2_msgs`、`foxglove_msgs`（自 [Foxglove schemas](https://github.com/foxglove/foxglove-sdk) 迁入，包名为 `foxglove_msgs.msg.v1`）。
