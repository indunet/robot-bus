# Rust API 示例

`Cargo.toml`：

```toml
robot-bus = "0.0.3"
# 本地：robot-bus = { path = "../robot-bus" }
# 默认已启用 gRPC；若需关闭：robot-bus = { version = "0.0.3", default-features = false }
```

## Broker 启动

SDK 默认连本机 broker（`localhost` + `tcp`）。运行示例前需先启动 `robot_bus_broker`；也可在应用内嵌 broker（见下文「进程内 broker」）。

| 组件 | 默认端口 | 说明 |
|------|----------|------|
| message | 15560 (XSUB) / 15561 (XPUB) | PUB/SUB 透明代理 |
| service | 15662 (frontend) / 15663 (backend) | REQ 客户端 ↔ DEALER worker |
| action | 15664 (frontend) / 15665 (backend) | DEALER 客户端 ↔ DEALER worker |
| gRPC | 15770 | gRPC + gRPC-Web（与总线同进程启动） |

**启动（三条总线 + gRPC）：**

```bash
cargo run --bin robot_bus_broker
# Ctrl+C 停止
# 查看全部参数：cargo run --bin robot_bus_broker -- --help
```

常用配置示例：

```bash
cargo run --bin robot_bus_broker -- \
  --message-xsub-bind tcp://0.0.0.0:15560 \
  --message-xpub-bind tcp://0.0.0.0:15561 \
  --service-frontend-bind tcp://0.0.0.0:15662 \
  --service-backend-bind tcp://0.0.0.0:15663 \
  --action-frontend-bind tcp://0.0.0.0:15664 \
  --action-backend-bind tcp://0.0.0.0:15665 \
  --grpc-listen 0.0.0.0:15770 \
  --snd-hwm 8 --rcv-hwm 8 \
  --tcp-only
```

**进程内嵌入**（不必单独起二进制）：

```rust
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::broker::{GrpcBrokerConfig, RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig {
    message: BusConfig {
        xsub_bind: "tcp://127.0.0.1:15560".into(),
        xpub_bind: "tcp://127.0.0.1:15561".into(),
        ..BusConfig::default()
    },
    service: ServiceBusConfig {
        frontend_bind: "tcp://127.0.0.1:15662".into(),
        backend_bind: "tcp://127.0.0.1:15663".into(),
        ..ServiceBusConfig::default()
    },
    grpc: GrpcBrokerConfig {
        listen: "0.0.0.0:15770".parse()?,
        ..GrpcBrokerConfig::default()
    },
    ..RobotBusConfig::default()
})?;
// broker.message.xsub_bind / xpub_bind 等填入 NodeOptions
// broker.grpc_listen() → gRPC 客户端连这个地址
broker.stop()?;
```

连接由 [`Node`](../src/runtime/node.rs) 的 `NodeOptions` 管理。典型流程：`Node::new` → `executor.add_node` → `create_*` → `executor.spin`。

---

## Message bus（Executor + Node + spin）

接近 ROS 2：先 `Node::new`，再 `executor.add_node`，然后 `create_publisher(topic)` 得到 publisher 再 `publish`。推荐 typed 订阅；也可传 raw `&[u8]` 回调自行 decode。

```rust
use std::sync::Arc;
use std::time::Duration;
use prost::Message;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::{Node, SingleThreadedExecutor};

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("pilot");
    // 进程内 / 自定义地址：Node::with_options("pilot", NodeOptions { ... })
    let executor = SingleThreadedExecutor::new();
    executor.add_node(&mut node)?;

    let imu_pub = node.create_publisher("/robot1/imu")?;
    node.create_subscription_typed::<Imu, _>(
        "/robot1/imu",
        |topic, imu| {
            println!("{topic}: angular_z={:?}", imu.angular_velocity);
        },
        None,
    )?;

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
    imu_pub.publish(&imu.encode_to_vec())?;

    node.create_timer(
        Duration::from_millis(100),
        Arc::new(|| {
            // 周期任务
        }),
        None,
    )?;

    let handle = executor.shutdown_handle()?;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        handle.shutdown();
    });

    executor.spin()?; // 阻塞
    Ok(())
}
```

Raw bytes 回调：`node.create_subscription("/robot1/imu", Arc::new(|topic, payload| { ... }), None)?`。

topic / service / action 名按传入原样使用（请自行写全路径）。

`MultiThreadedExecutor::new(n)`：最多 `n` 个 worker；配合 `Reentrant` callback group 时订阅 / timer / service / action 都可并行。`MutuallyExclusive` 组内仍串行。

### Callback group

接近 ROS 2：`MutuallyExclusive`（组内互斥）与 `Reentrant`（组内可并行，配合 `MultiThreadedExecutor`）。节点默认有一个互斥 group；`callback_group` 传 `None` 时用默认组（与 ROS 2 把 group 作为参数传入一致）。

```rust
use robot_bus::{CallbackGroupType, MultiThreadedExecutor, Node};

let mut node = Node::new("pilot");
let executor = MultiThreadedExecutor::new(4);
executor.add_node(&mut node)?;

let reentrant = node.create_callback_group(CallbackGroupType::Reentrant);
node.create_subscription(
    "/robot1/imu",
    Arc::new(|_topic, _payload| { /* 可与同组其它回调并行 */ }),
    Some(&reentrant),
)?;
node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| {}),
    Some(&reentrant),
)?;
```


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
node.create_service("echo", handler, /* identity */ None, /* callback_group */ None)?;
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
use robot_bus::{Node, NodeOptions, RobotBusBroker, RobotBusConfig, SingleThreadedExecutor};

fn main() -> anyhow::Result<()> {
    let broker = RobotBusBroker::start(RobotBusConfig::default())?;

    let options = NodeOptions {
        message_xsub: Some(broker.message.xsub_bind.clone()),
        message_xpub: Some(broker.message.xpub_bind.clone()),
        ..NodeOptions::default()
    };
    let mut node = Node::with_options("demo", options);
    let executor = SingleThreadedExecutor::new();
    executor.add_node(&mut node)?;
    let imu_pub = node.create_publisher("/robot1/imu")?;
    node.create_subscription(
        "/robot1/imu",
        Arc::new(|topic, payload| println!("{topic}: {} bytes", payload.len())),
        None,
    )?;
    imu_pub.publish(b"hello")?;
    executor.spin_once(None)?;

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

## gRPC / gRPC-Web 网关

由 `RobotBusBroker` / `robot_bus_broker` 一并启动（feature `grpc`，默认开启）。标准 gRPC 与 gRPC-Web **同端口**（默认 `0.0.0.0:15770`）。

| RPC | 说明 |
|-----|------|
| `MessageGateway.Subscribe` | server stream：topic 前缀 → `TopicMessage` |
| `ServiceGateway.Call` | 一元：`service_name` + request bytes → response bytes |
| `ActionGateway.Run` | 双向流：客户端 `GoalCommand` / `CancelCommand` ↔ 服务端 `ActionEvent` |

Subscribe 示例：

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

Service / Action 示例：

```rust
use robot_bus::grpc::pb::action_gateway_client::ActionGatewayClient;
use robot_bus::grpc::pb::service_gateway_client::ServiceGatewayClient;
use robot_bus::grpc::pb::{
    action_client_message, ActionClientMessage, ActionKind, GoalCommand, ServiceCallRequest,
};
use tonic::Request;
use tokio_stream::{self, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut svc = ServiceGatewayClient::connect("http://127.0.0.1:15770").await?;
    let resp = svc
        .call(Request::new(ServiceCallRequest {
            service_name: "svc.echo".into(),
            request: b"ping".to_vec(),
            request_id: String::new(),
            timeout_ms: 5_000,
        }))
        .await?
        .into_inner();
    println!("service: {} bytes", resp.response.len());

    let mut act = ActionGatewayClient::connect("http://127.0.0.1:15770").await?;
    let outbound = tokio_stream::iter(vec![ActionClientMessage {
        msg: Some(action_client_message::Msg::Goal(GoalCommand {
            action_name: "act.demo".into(),
            goal: b"go".to_vec(),
            goal_id: String::new(),
            timeout_ms: 10_000,
        })),
    }]);
    let mut stream = act.run(Request::new(outbound)).await?.into_inner();
    while let Some(ev) = stream.next().await {
        let ev = ev?;
        println!("{:?}: {} bytes", ActionKind::try_from(ev.kind), ev.body.len());
    }
    Ok(())
}
```

Proto 包名：`robot_bus.grpc.v1`。见 `proto/robot_bus/grpc/v1/{message,service,action}_gateway.proto`。

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
