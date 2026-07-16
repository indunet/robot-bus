# Rust API 示例

`Cargo.toml`：

```toml
robot-bus = "0.0.2"
# 本地：robot-bus = { path = "../robot-bus" }
# gRPC 网关：robot-bus = { version = "0.0.2", features = ["grpc"] }
```

## Broker 启动

SDK 默认连本机 broker（`localhost` + `tcp`）。运行示例前需先启动对应路由进程；也可在应用内嵌 broker（见下文「进程内 broker」）。

| 总线 | 默认端口 | 二进制 | 说明 |
|------|----------|--------|------|
| message | 15560 (XSUB) / 15561 (XPUB) | `message_bus_broker` | PUB/SUB 透明代理 |
| service | 15662 (frontend) / 15663 (backend) | `service_bus_broker` | REQ 客户端 ↔ DEALER worker |
| action | 15664 (frontend) / 15665 (backend) | `action_bus_broker` | DEALER 客户端 ↔ DEALER worker |

**一次启动三条总线（最常用）：**

```bash
cargo run --bin robot_bus_broker
# Ctrl+C 停止
```

**只启动某一条总线：**

```bash
cargo run --bin message_bus_broker
cargo run --bin service_bus_broker
cargo run --bin action_bus_broker
# 各二进制支持 --help 查看 bind / HWM 等参数
```

**gRPC 网关**（feature `grpc`，订阅 message topic；需 message broker 已运行）：

```bash
cargo run --bin robot_bus_broker
cargo run --features grpc --bin robot_bus_grpc_gateway
# 默认 http://0.0.0.0:15770
```

**进程内嵌入**（不必单独起二进制）：

```rust
use robot_bus::broker::{RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig::default())?;
// broker.message.xsub_bind / xpub_bind 等填入 NodeOptions
broker.stop()?;
```

连接由 [`Node`](../src/runtime/node.rs) 的 `NodeOptions` 管理。底层 `Executor` 负责 `spin`；一般业务代码用 Node 即可。

---

## Message bus（Node + spin）

接近 ROS 2：`create_publisher` / `create_subscription` / `spin`。推荐 typed 订阅（标准 `sensor_msgs::msg::v1::Imu`）；也可传 raw `&[u8]` 回调自行 decode。

```rust
use std::sync::Arc;
use std::time::Duration;
use prost::Message;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::with_namespace("pilot", "robot1");
    // 默认连本机 broker；进程内 / 自定义地址用 Node::with_options(..., NodeOptions { ... })
    node.create_publisher()?;
    node.create_subscription_typed::<Imu, _>("imu", |topic, imu| {
        // 实际 topic: robot1/imu
        println!("{topic}: angular_z={:?}", imu.angular_velocity);
    })?;

    let imu = Imu {
        angular_velocity: Some(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.1,
        }),
        linear_acceleration: Some(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 9.8,
        }),
        ..Default::default()
    };
    node.publish("imu", &imu.encode_to_vec())?;

    node.create_timer(Duration::from_millis(100), Arc::new(|| {
        // 周期任务
    }))?;

    let handle = node.shutdown_handle();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        handle.shutdown();
    });

    node.spin()?; // 阻塞
    // 或：node.spin_once(Some(Duration::from_millis(100)))?;
    Ok(())
}
```

Raw bytes 回调：`node.create_subscription("imu", Arc::new(|topic, payload| { ... }))?`。

绝对名以 `/` 开头时不加命名空间前缀。

`Node::with_worker_pool(n)` / `with_options_and_pool`：service / action handler 最多 `n` 个并发线程；订阅与 timer 仍在 I/O 线程。底层对应 `Executor::with_worker_pool(n)`。

### 高水位（HWM）

```rust
use robot_bus::{Publisher, HighWaterMark};

let pub_ = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
pub_.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

---

## Service bus

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::service_frontend_endpoint;

fn main() -> robot_bus::Result<()> {
    let frontend = service_frontend_endpoint("localhost", "tcp")?;
    let backend = robot_bus::service_backend_endpoint("localhost", "tcp")?;

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| [b"echo:", body].concat());

    let worker = WorkerThread::spawn_service("svc.echo", handler, &backend)?;
    std::thread::sleep(Duration::from_millis(100));

    let client = ServiceClient::new(Some(&frontend))?;
    let reply = client.call("svc.echo", b"ping", None, Some(Duration::from_secs(10)))?;
    assert_eq!(reply, b"echo:ping");

    worker.stop();
    Ok(())
}
```

也可用 `Node::create_service` + `spin`（endpoint 取自 `NodeOptions`）：

```rust
node.create_service("echo", handler, /* identity */ None)?;
```

---

## Action bus

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::action_bus::{ActionClient, ActionKind};
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{action_backend_endpoint, action_frontend_endpoint};

fn main() -> robot_bus::Result<()> {
    let frontend = action_frontend_endpoint("localhost", "tcp")?;
    let backend = action_backend_endpoint("localhost", "tcp")?;

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|_client_id, _goal_id, body| {
            vec![
                ("FEEDBACK".into(), b"step-1".to_vec()),
                ("RESULT".into(), [b"done:", body].concat()),
            ]
        });

    let worker = WorkerThread::spawn_action("act.demo", handler, &backend)?;
    std::thread::sleep(Duration::from_millis(100));

    let client = ActionClient::new(Some(&frontend))?;
    let messages = client.send_goal("act.demo", b"fly", None, Some(Duration::from_secs(30)))?;

    for msg in &messages {
        match msg.kind {
            ActionKind::Feedback => println!("feedback: {:?}", msg.body),
            ActionKind::Result => println!("result: {:?}", msg.body),
            _ => {}
        }
    }

    worker.stop();
    Ok(())
}
```

Node：`create_action` / `connect_action_client` 同样使用 `NodeOptions` 里的 action endpoint。

---

## 进程内 broker

不必单独起 `robot_bus_broker` 二进制；用 `NodeOptions` 填入 broker 返回的 bind 地址：

```rust
use std::sync::Arc;
use robot_bus::{Node, NodeOptions, RobotBusBroker, RobotBusConfig};

fn main() -> anyhow::Result<()> {
    let broker = RobotBusBroker::start(RobotBusConfig::default())?;

    let options = NodeOptions {
        message_xsub: Some(broker.message.xsub_bind.clone()),
        message_xpub: Some(broker.message.xpub_bind.clone()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("demo", "", options);
    node.create_publisher()?;
    node.create_subscription(
        "imu",
        Arc::new(|topic, payload| println!("{topic}: {} bytes", payload.len())),
    )?;
    node.publish("imu", b"hello")?;
    node.spin_once(None)?;

    broker.stop()?;
    Ok(())
}
```

---

## Protobuf 消息（`robot_bus::<pkg>`）

总线仍传 opaque bytes。消息类型挂在 crate 命名空间下，例如 `robot_bus::sensor_msgs::msg::v1::Imu`（无中间 `msgs` 层）。`create_subscription_typed` 会自动 decode；raw 回调则自行 `Message::decode`。其它消息同理，例如 `geometry_msgs::msg::v1::Twist`：

```rust
use prost::Message;
use robot_bus::geometry_msgs::msg::v1::{Twist, Vector3};

let twist = Twist {
    linear: Some(Vector3 { x: 1.0, y: 0.0, z: 0.0 }),
    angular: Some(Vector3::default()),
};
node.publish("cmd_vel", &twist.encode_to_vec())?;
```

Service 的 Request / Response 同理（如 `robot_bus::std_srvs::srv::v1::SetBoolRequest`）。

---

## gRPC / gRPC-Web 网关（feature `grpc`）

独立进程，浏览器或原生 gRPC 客户端订阅 message topic（启动方式见上文「Broker 启动」）。

Rust 客户端示例（集成测试同款）：

```rust
use robot_bus::grpc::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::grpc::pb::SubscribeRequest;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessageGatewayClient::connect("http://127.0.0.1:15770").await?;
    let mut stream = client
        .subscribe(Request::new(SubscribeRequest {
            topic: "imu".into(),
        }))
        .await?
        .into_inner();

    use tokio_stream::StreamExt;
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        println!("{}: {} bytes", msg.topic, msg.payload.len());
    }
    Ok(())
}
```

Proto 包名：`robot_bus.grpc.v1`（与 ROS `*.msg.v1` / `*.srv.v1` 区分）。见 `proto/robot_bus/grpc/v1/message_gateway.proto`。

---

## 传输与端点

一般由 `NodeOptions` 推导；需要手工拼地址时：

```rust
use robot_bus::transports::{
    message_xsub_endpoint, message_xpub_endpoint,
    service_frontend_endpoint, service_backend_endpoint,
    action_frontend_endpoint, action_backend_endpoint,
};

// transport: "tcp" | "ipc" | "inproc"
let ep = message_xpub_endpoint("localhost", "tcp")?;
```

---

## 错误类型

```rust
use robot_bus::{BusError, Result};

match result {
    Err(BusError::Timeout(_)) => { /* ... */ }
    Err(BusError::NoWorker { name }) => { /* service/action 无 worker */ }
    Err(e) => eprintln!("{e}"),
    Ok(v) => { /* ... */ }
}
```
