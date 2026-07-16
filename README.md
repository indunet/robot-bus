# Robot Bus

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
| `runtime::SingleThreadedExecutor` / `MultiThreadedExecutor` | ROS 2 风格执行器；`add_node` + `spin` |
| `runtime::Node` / `TopicPublisher` | 节点 + `create_publisher(topic)` 返回的 publisher |
| `grpc::`（feature） | gRPC / gRPC-Web 网关（Subscribe / Call / Run） |
| [`proto/`](proto/) | ROS 风格 Protobuf：`proto/<pkg>/{msg\|srv\|grpc}/v1/` → Rust/Python `robot_bus.<pkg>…` |

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
maturin develop --features extension-module
```

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, payload):
    imu = Imu()
    imu.ParseFromString(payload)
    print(topic, imu.linear_acceleration)

node = robot_bus.Node("pilot")
executor = robot_bus.SingleThreadedExecutor()
executor.add_node(node)

imu_pub = node.create_publisher("/robot1/imu")
node.create_subscription("/robot1/imu", on_imu)
imu_pub.publish(
    Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8)).SerializeToString(),
)
# executor.spin()  # 阻塞直到其它线程调用 executor.shutdown()
```

### 2. Rust（Executor + Node + spin）

在 `Cargo.toml` 中添加依赖：

```toml
robot-bus = { path = "../robot-bus" }
# 或 crates.io：robot-bus = "0.0.2"
```

语义接近 ROS 2：`Node::new` → `executor.add_node` → `create_publisher(topic)` → `spin`：

```rust
use std::sync::Arc;
use std::time::Duration;
use prost::Message;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::{Node, SingleThreadedExecutor};

let mut node = Node::new("pilot");
let executor = SingleThreadedExecutor::new();
executor.add_node(&mut node)?;

let imu_pub = node.create_publisher("/robot1/imu")?;
node.create_subscription_typed::<Imu, _>("/robot1/imu", |topic, imu| {
    println!("{topic}: {:?}", imu.linear_acceleration);
})?;

let imu = Imu {
    linear_acceleration: Some(Vector3 { x: 0.0, y: 0.0, z: 9.8 }),
    ..Default::default()
};
imu_pub.publish(&imu.encode_to_vec())?;

node.create_timer(Duration::from_millis(100), Arc::new(|| {
    // 控制周期 / 心跳
}))?;

let handle = executor.shutdown_handle()?;
std::thread::spawn(move || { /* ... */ handle.shutdown(); });
executor.spin()?;
```

- `SingleThreadedExecutor`：回调在 I/O / spin 线程串行（默认）
- `MultiThreadedExecutor::new(n)`：service / action handler 最多 `n` 个并发线程；订阅与 timer 仍在 I/O 线程
- Raw bytes：`create_subscription(topic, Arc::new(|topic, payload| { ... }))`
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
| `robot_bus_broker` | 一次启动三个 bus |
| `message_bus_broker` | 仅 message bus |
| `service_bus_broker` | 仅 service bus |
| `action_bus_broker` | 仅 action bus |
| `robot_bus_grpc_gateway` | gRPC / gRPC-Web 网关（需 `--features grpc`） |

## gRPC / gRPC-Web 网关

独立进程，连已有 message / service / action bus；标准 gRPC 与 gRPC-Web **同端口**（默认 `0.0.0.0:15770`）。

| RPC | 语义 |
|-----|------|
| `MessageGateway.Subscribe` | 按 topic 前缀订阅，服务端流式返回二进制 payload |
| `ServiceGateway.Call` | 一元：`service_name` + request bytes → response bytes |
| `ActionGateway.Run` | 双向流：客户端发 GOAL / CANCEL，服务端推 `ActionEvent`（`kind` 区分 FEEDBACK / RESULT） |

```bash
cargo run --bin robot_bus_broker
cargo run --features grpc --bin robot_bus_grpc_gateway
# --listen 0.0.0.0:15770
# --message-xpub tcp://127.0.0.1:15561
# --service-frontend tcp://127.0.0.1:15662
# --action-frontend tcp://127.0.0.1:15664
# --cors-origin http://localhost:3000   # 可重复；默认允许任意 origin
```

Proto（包名 `robot_bus.grpc.v1`，与 ROS `*.msg.v1` / `*.srv.v1` 区分）：

- [`message_gateway.proto`](proto/robot_bus/grpc/v1/message_gateway.proto)
- [`service_gateway.proto`](proto/robot_bus/grpc/v1/service_gateway.proto)
- [`action_gateway.proto`](proto/robot_bus/grpc/v1/action_gateway.proto)

## 测试

```bash
cargo test
cargo test --features grpc
PYTHONPATH=python python3 tests/python/test_msgs_roundtrip.py
```

## Protobuf 消息

[`proto/`](proto/) 按 ROS 包布局：`proto/<pkg>/{msg|srv|grpc}/v1/*.proto`。

| 语言 | 路径 | 说明 |
|------|------|------|
| Rust | `robot_bus::<pkg>::{msg\|srv}::v1` | `build.rs` + prost；挂在 crate 命名空间下 |
| Python | `robot_bus.<pkg>.{msg\|srv}.v1` | 随 wheel 打包；`scripts/generate_python_msgs.py` 生成 |

- 传输层 body 仍是 opaque bytes；bus 不解析类型，业务侧自行 `encode` / `decode`（或用 `create_subscription_typed`）
- **srv** 是一对 `*Request` / `*Response` message，不是 gRPC
- **grpc**（`robot_bus`）是网关 RPC 契约，走 feature `grpc` + tonic
- 消息在 `robot_bus` 命名空间下，**不占用** ROS 顶层 `sensor_msgs` 包名；编码是 protobuf，与 ROS CDR 不互通
- 改 proto 后跑：`python3 scripts/generate_python_msgs.py`（建议 protoc 28.x，与 CI 一致）

已覆盖：`builtin_interfaces`、`std_msgs`、`std_srvs`、`geometry_msgs`、`sensor_msgs`、`nav_msgs`、`tf2_msgs`、`trajectory_msgs`、`diagnostic_msgs`、`unique_identifier_msgs`、`shape_msgs`、`visualization_msgs`、`control_msgs`、`nav2_msgs`。
