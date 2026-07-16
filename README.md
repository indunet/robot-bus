# Robot Bus

轻量级、免环境配置的 ROS 2 风格通信库：基于 ZeroMQ，提供 topic / service / action，以及 `Node` + `spin` 回调模型。

不依赖 ROS 发行版、不需要 `source setup.bash`、不搭 workspace。一个 broker 进程 + SDK（Rust / Python）即可。

更多 API 示例见 [`docs/`](docs/)。

| 模块 | 职责 |
|------|------|
| `broker::` | 路由进程（message / service / action） |
| 顶层 API | Publisher / Subscriber / Client / Worker |
| `runtime::BusRuntime` | 回调 executor（`spin` / `spin_once`） |
| `runtime::Node` | Node 门面（名字 / 命名空间 + `create_*`） |
| `grpc::`（feature） | gRPC / gRPC-Web 网关（独立进程，先支持 Subscribe） |
| [`proto/`](proto/) | 常用 ROS 2 消息 / Service 的 Protobuf 定义 |

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
from sensor_msgs_pb2 import Imu
from geometry_msgs_pb2 import Vector3

def on_imu(topic, payload):
    imu = Imu()
    imu.ParseFromString(payload)
    print(topic, imu.linear_acceleration)

node = robot_bus.Node("pilot", namespace="robot1")
node.create_publisher(robot_bus.message_xsub_endpoint())
node.create_subscription("imu", on_imu, robot_bus.message_xpub_endpoint())
node.publish(
    "imu",
    Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8)).SerializeToString(),
)
# node.spin()  # 阻塞直到其它线程调用 node.shutdown()
```

### 2. Rust（回调 / spin）

在 `Cargo.toml` 中添加依赖：

```toml
robot-bus = { path = "../robot-bus" }
# 或 crates.io：robot-bus = "0.0.2"
```

语义接近 ROS 2 的 `spin` / `spin_once` / `spin_some`：

```rust
use std::sync::Arc;
use prost::Message;
use robot_bus::msgs::geometry_msgs::v1::Vector3;
use robot_bus::msgs::sensor_msgs::v1::Imu;
use robot_bus::{
    BusRuntime, MessageCallback, Publisher, message_xpub_endpoint, message_xsub_endpoint,
};

let pub_ = Publisher::new(Some(&message_xsub_endpoint("localhost", "tcp")?))?;
let mut rt = BusRuntime::new();
rt.connect_subscriber(Some(&message_xpub_endpoint("localhost", "tcp")?))?;

let cb: MessageCallback = Arc::new(|topic, payload| {
    let imu = Imu::decode(payload).expect("decode Imu");
    println!("{topic}: {:?}", imu.linear_acceleration);
});
rt.subscribe("imu", cb)?;

let imu = Imu {
    linear_acceleration: Some(Vector3 { x: 0.0, y: 0.0, z: 9.8 }),
    ..Default::default()
};
pub_.publish("imu", &imu.encode_to_vec())?;

rt.create_timer(Duration::from_millis(100), Arc::new(|| {
    // 控制周期 / 心跳
}))?;

let handle = rt.shutdown_handle();
std::thread::spawn(move || { /* ... */ handle.shutdown(); });
rt.spin()?;
```

- `BusRuntime::new()`：回调在 I/O / spin 线程执行
- `BusRuntime::with_executor(n)`：service / action handler 最多 `n` 个并发线程；订阅与 timer 仍在 I/O 线程

发送 / 接收水位（ZMQ HWM，不是完整 QoS）可在创建时或运行中设置：

```rust
use robot_bus::{Publisher, HighWaterMark};

let pub_ = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
pub_.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

默认：message `STREAM(2/2)`、service `RPC(4/4)`、action `ACTION(8/8)`。Broker 侧用 `--snd-hwm` / `--rcv-hwm`。

### 3. Node 门面

`Node` 包一层 `BusRuntime`，API 接近 ROS 2：`create_subscription` / `create_publisher` / `create_timer` / `create_service`。相对名会加命名空间前缀；以 `/` 开头的绝对名不变。

```rust
use std::sync::Arc;
use robot_bus::{Node, message_xpub_endpoint, message_xsub_endpoint};

let mut node = Node::with_namespace("pilot", "robot1");
node.create_publisher(Some(&message_xsub_endpoint("localhost", "tcp")?))?;
node.create_subscription(
    "imu", // → robot1/imu
    Arc::new(|topic, payload| println!("{topic}: {} bytes", payload.len())),
    Some(&message_xpub_endpoint("localhost", "tcp")?),
)?;
node.create_timer(Duration::from_millis(100), Arc::new(|| { /* ... */ }))?;
node.publish("cmd_vel", twist_bytes)?; // → robot1/cmd_vel
node.spin()?;
```

## 二进制

| 二进制 | 说明 |
|------|------|
| `robot_bus_broker` | 一次启动三个 bus |
| `message_bus_broker` | 仅 message bus |
| `service_bus_broker` | 仅 service bus |
| `action_bus_broker` | 仅 action bus |
| `robot_bus_grpc_gateway` | gRPC / gRPC-Web 网关（需 `--features grpc`） |

## gRPC / gRPC-Web 网关

独立进程，连已有 message bus XPUB；按请求里的 topic 前缀订阅，服务端流式返回二进制 payload。标准 gRPC 与 gRPC-Web **同端口**（默认 `0.0.0.0:15770`）。后续可在同一网关扩展 Service / Action。

```bash
cargo run --bin robot_bus_broker
cargo run --features grpc --bin robot_bus_grpc_gateway
# --listen 0.0.0.0:15770
# --message-xpub tcp://127.0.0.1:15561
# --cors-origin http://localhost:3000   # 可重复；默认允许任意 origin
```

Proto：[`proto/robot_bus/grpc/v1/message_gateway.proto`](proto/robot_bus/grpc/v1/message_gateway.proto)（包名 `robot_bus.grpc.v1`，与 ROS msg/srv 的 `*.v1` 区分）— `MessageGateway.Subscribe`。

## 测试

```bash
cargo test
cargo test --features grpc
```

## Protobuf 消息

[`proto/`](proto/) 是常用 ROS 2 **msg** / **srv** 的 Protobuf 重定义，经 `build.rs` + prost 生成到 `robot_bus::msgs`。

- 传输层 body 仍是 opaque bytes；bus 不解析类型，业务侧自行 `encode` / `decode`
- **srv** 是一对 `*Request` / `*Response` message，不是 gRPC

已覆盖：`builtin_interfaces`、`std_msgs`、`std_srvs`、`geometry_msgs`、`sensor_msgs`、`nav_msgs`、`tf2_msgs`、`trajectory_msgs`、`diagnostic_msgs`、`unique_identifier_msgs`、`shape_msgs`、`visualization_msgs`、`control_msgs`、`nav2_msgs`。
