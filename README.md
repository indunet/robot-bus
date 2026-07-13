# Robot Bus

统一的 ZeroMQ 消息总线：broker 路由 + 参与方 SDK，单一 Rust crate。

| 模块 | 职责 |
|------|------|
| `broker::` | 路由进程（message / service / action 三种 bus） |
| 顶层 API | Publisher / Subscriber / Client / Worker |

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
