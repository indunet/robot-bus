# Robot Bus

统一的 ZeroMQ 消息总线：broker 路由 + 参与方 SDK，单一 Rust crate。

| 模块 | 职责 |
|------|------|
| `broker::` | 路由进程（message / service / action 三种 bus） |
| 顶层 API | Publisher / Subscriber / Client / Worker |
| [`proto/`](proto/) | ROS 2 标准消息 / Service 的 Protobuf 定义 |

## 架构

```
业务模块 (pilot / wireless-sensor / controller / robot-hmi / ...)
  └── Cargo path → robot-bus
              │
              │ ZMQ (tcp / ipc / inproc)
              ▼
robot_bus_broker 进程  (cargo build)
```

## 快速开始

### 1. 启动 broker

```bash
cargo run --bin robot_bus_broker
```

### 2. 业务代码

在 `Cargo.toml` 中添加 path 依赖：

```toml
robot_bus = { path = "../robot-bus" }
```

```rust
use robot_bus::{Publisher, Subscriber, message_xsub_endpoint, message_xpub_endpoint};

let pub_ = Publisher::new(None)?;
pub_.publish("wireless.imu", imu_bytes)?;
let sub = Subscriber::new(Some(&message_xpub_endpoint("localhost", "tcp")?))?;
sub.subscribe("wireless.imu")?;
```

## 二进制

| 二进制 | 说明 |
|--------|------|
| `robot_bus_broker` | 一次启动三个 bus |
| `message_bus_broker` | 仅 message bus |
| `service_bus_broker` | 仅 service bus |
| `action_bus_broker` | 仅 action bus |

## 测试

```bash
cargo test
```

## Protobuf（ROS 2 消息 / Service）

[`proto/`](proto/) 下是 ROS 2 常用 **msg** 与 **srv** 的 Protobuf 重定义，经 `build.rs` + prost 生成到 `robot_bus::msgs`。

- **msg**：普通 message（如 `Twist`、`Odometry`）
- **srv**：一对 `*Request` / `*Response` message（如 `std_srvs::SetBoolRequest`），**不是 gRPC**（无 `service`/`rpc` 块，也不走 HTTP/2）
- 传输层 body 仍是 opaque bytes；message bus / service bus 都不解析类型，业务侧自行 `encode` / `decode`

已覆盖包：`builtin_interfaces`、`std_msgs`、`std_srvs`、`geometry_msgs`、`sensor_msgs`、`nav_msgs`（含 GetMap / SetMap / GetPlan）、`tf2_msgs`、`trajectory_msgs`、`diagnostic_msgs`、`unique_identifier_msgs`、`shape_msgs`、`visualization_msgs`、`control_msgs`、`nav2_msgs`。
