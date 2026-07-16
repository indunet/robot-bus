# Rust API 示例

`Cargo.toml`：

```toml
robot-bus = "0.0.2"
# 本地：robot-bus = { path = "../robot-bus" }
# gRPC 网关：robot-bus = { version = "0.0.2", features = ["grpc"] }
```

先启动 broker：`cargo run --bin robot_bus_broker`。

---

## Message bus（`BusRuntime` / 回调）

接近 ROS 2：订阅时注册回调，由 `spin` / `spin_once` 驱动；payload 为不透明 `&[u8]`，业务侧用 `robot_bus::msgs` encode / decode（此处用标准 `sensor_msgs::v1::Imu`）：

```rust
use std::sync::Arc;
use std::time::Duration;
use prost::Message;
use robot_bus::msgs::geometry_msgs::v1::Vector3;
use robot_bus::msgs::sensor_msgs::v1::Imu;
use robot_bus::{
    BusRuntime, MessageCallback, Publisher, message_xpub_endpoint, message_xsub_endpoint,
};

fn main() -> robot_bus::Result<()> {
    let pub_ = Publisher::new(Some(&message_xsub_endpoint("localhost", "tcp")?))?;

    let mut rt = BusRuntime::new();
    rt.connect_subscriber(Some(&message_xpub_endpoint("localhost", "tcp")?))?;

    let cb: MessageCallback = Arc::new(|topic, payload| {
        let imu = Imu::decode(payload).expect("decode Imu");
        println!("{topic}: angular_z={:?}", imu.angular_velocity);
    });
    rt.subscribe("imu", cb)?;

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
    pub_.publish("imu", &imu.encode_to_vec())?;

    rt.create_timer(Duration::from_millis(100), Arc::new(|| {
        // 周期任务
    }))?;

    // 另一线程触发退出
    let handle = rt.shutdown_handle();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        handle.shutdown();
    });

    rt.spin()?; // 阻塞
    // 或嵌入自己的循环：rt.spin_once(Some(Duration::from_millis(100)))?;
    Ok(())
}
```

`BusRuntime::with_executor(n)`：service / action handler 最多 `n` 个并发线程；订阅与 timer 仍在 I/O 线程。

### 高水位（HWM）

```rust
use robot_bus::{Publisher, HighWaterMark};

let pub_ = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
pub_.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

---

## Node 门面

带名字与命名空间，相对 topic 自动加前缀：

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::{Node, message_xpub_endpoint, message_xsub_endpoint};

fn main() -> robot_bus::Result<()> {
    let mut node = Node::with_namespace("pilot", "robot1");
    node.create_publisher(Some(&message_xsub_endpoint("localhost", "tcp")?))?;
    node.create_subscription(
        "imu", // → robot1/imu
        Arc::new(|topic, payload| println!("{topic}: {} bytes", payload.len())),
        Some(&message_xpub_endpoint("localhost", "tcp")?),
    )?;
    node.create_timer(Duration::from_millis(100), Arc::new(|| {}))?;

    node.publish("cmd_vel", b"...")?; // → robot1/cmd_vel
  // node.spin()?;
    Ok(())
}
```

绝对名以 `/` 开头时不加命名空间前缀。

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

也可用 `Node::create_service` + `spin` 在回调模型里注册 handler（见 `runtime::node`）。

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

---

## 进程内 broker

不必单独起 `robot_bus_broker` 二进制，可在应用内启动：

```rust
use std::sync::Arc;
use robot_bus::{Node, RobotBusBroker, RobotBusConfig};

fn main() -> anyhow::Result<()> {
    let broker = RobotBusBroker::start(RobotBusConfig::default())?;

    let mut node = Node::new("demo");
    node.create_publisher(Some(&broker.message.xsub_bind))?;
    node.create_subscription(
        "imu",
        Arc::new(|topic, payload| println!("{topic}: {} bytes", payload.len())),
        Some(&broker.message.xpub_bind),
    )?;
    node.publish("imu", b"hello")?;
    node.spin_once(None)?;

    broker.stop()?;
    Ok(())
}
```

---

## Protobuf 消息（`robot_bus::msgs`）

总线仍传 opaque bytes；在 publish 前 encode、在订阅回调里 decode。上面示例用的是标准 `sensor_msgs::v1::Imu`（见 `proto/sensor_msgs/v1/imu.proto`）。其它消息同理，例如 `geometry_msgs::v1::Twist`：

```rust
use prost::Message;
use robot_bus::msgs::geometry_msgs::v1::{Twist, Vector3};

let twist = Twist {
    linear: Some(Vector3 { x: 1.0, y: 0.0, z: 0.0 }),
    angular: Some(Vector3::default()),
};
node.publish("cmd_vel", &twist.encode_to_vec())?;

// 订阅回调里：
// let decoded = Twist::decode(payload)?;
```

Service 的 Request / Response 同理（如 `std_srvs::v1::SetBoolRequest`）。

---

## gRPC / gRPC-Web 网关（feature `grpc`）

独立进程，浏览器或原生 gRPC 客户端订阅 message topic：

```bash
cargo run --bin robot_bus_broker
cargo run --features grpc --bin robot_bus_grpc_gateway
# 默认 http://0.0.0.0:15770
```

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

Proto 包名：`robot_bus.grpc.v1`（与 ROS msg/srv 的 `*.v1` 区分）。见 `proto/robot_bus/grpc/v1/message_gateway.proto`。

---

## 传输与端点

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
