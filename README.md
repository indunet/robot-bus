# Robot Bus

统一的 ZeroMQ 消息总线：broker 路由 + 参与方 SDK，单一 Rust crate。

| 模块 | 职责 |
|------|------|
| `broker::` | 路由进程（message / service / action 三种 bus） |
| 顶层 API | Publisher / Subscriber / Client / Worker（拉取 / serve） |
| `runtime::BusRuntime` | ROS 2 风格回调 executor（`spin` / `spin_once`） |
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

### 2. 业务代码（拉取式）

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

### 3. 回调式（ROS 2 风格 spin）

注册回调后用 executor 驱动，语义接近 ROS 2 的 `spin` / `spin_once` / `spin_some`：

```rust
use std::sync::Arc;
use robot_bus::{BusRuntime, MessageCallback, message_xpub_endpoint};

let mut rt = BusRuntime::new(); // 单线程：回调跑在 spin 线程
rt.connect_subscriber(Some(&message_xpub_endpoint("localhost", "tcp")?))?;
let cb: MessageCallback = Arc::new(|topic, payload| {
    println!("{topic}: {} bytes", payload.len());
});
rt.subscribe("wireless.imu", cb)?;

// 定时器（同样由 spin 驱动）
rt.create_timer(Duration::from_millis(100), Arc::new(|| {
    // 控制周期 / 心跳 / 状态上报
}))?;

// 方式 A：阻塞直到别的线程调用 shutdown
let handle = rt.shutdown_handle();
std::thread::spawn(move || { /* ... */ handle.shutdown(); });
rt.spin()?;

// 方式 B：自己轮询（可嵌入其它循环）
// while running { rt.spin_once(Some(Duration::from_millis(100)))?; }

// 方式 C：后台线程
// rt.start()?;  /* ... */  rt.shutdown(); rt.wait();
```

- 默认 `BusRuntime::new()`：所有回调（含 timer）在 I/O / spin 线程执行（类似 SingleThreadedExecutor）
- `BusRuntime::with_executor(n)`：service / action handler 最多 `n` 个并发线程；订阅与 timer 回调仍在 I/O 线程；池满时回退到同步执行
- 也可只注册 timer、不连 subscriber，用 `spin` / `spin_once` 驱动
## 二进制

| 二进制 | 说明 |
|------|------|
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
