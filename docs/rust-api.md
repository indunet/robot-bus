# Rust API 示例

`Cargo.toml`：

```toml
robot-bus = "0.1.6"
# 本地：robot-bus = { path = "../robot-bus" }
# 默认已启用 gRPC；若需关闭：robot-bus = { version = "0.1.6", default-features = false }
```

## Broker 启动

运行示例前先起 broker（也可进程内嵌入，见下文）。默认一次启动 **message / service / action** 三条总线，每个 socket 同时 bind **tcp + ipc + inproc**，并启动 **grpc / grpc-web** 与 **console http**（控制台）。

```bash
cargo run --bin robot_bus_broker
# 查看参数：cargo run --bin robot_bus_broker -- --help
# 只要 TCP：加 --tcp-only
# 发现：--domain-id N / --advertise-host HOST / --no-discovery
```

**默认端点：**

| 角色 | tcp | ipc | inproc |
|------|-----|-----|--------|
| message XSUB | `tcp://0.0.0.0:15560` | `ipc:///tmp/robot_bus/message_bus_xsub.ipc` | `inproc://robot_bus/message_bus/xsub` |
| message XPUB | `tcp://0.0.0.0:15561` | `ipc:///tmp/robot_bus/message_bus_xpub.ipc` | `inproc://robot_bus/message_bus/xpub` |
| service frontend | `tcp://0.0.0.0:15662` | `ipc:///tmp/robot_bus/service_bus_frontend.ipc` | `inproc://robot_bus/service_bus/frontend` |
| service backend | `tcp://0.0.0.0:15663` | `ipc:///tmp/robot_bus/service_bus_backend.ipc` | `inproc://robot_bus/service_bus/backend` |
| action frontend | `tcp://0.0.0.0:15664` | `ipc:///tmp/robot_bus/action_bus_frontend.ipc` | `inproc://robot_bus/action_bus/frontend` |
| action backend | `tcp://0.0.0.0:15665` | `ipc:///tmp/robot_bus/action_bus_backend.ipc` | `inproc://robot_bus/action_bus/backend` |
| grpc | `0.0.0.0:15770` | — | — |
| grpc-web | `0.0.0.0:15770` | — | — |
| console http | `0.0.0.0:15770` | — | — |
| discovery (UDP multicast) | `239.255.76.67:15550` | — | — |

SDK 侧 `Node::new` 默认连本机 **tcp**（`localhost` + 上表端口）；`Node::ipc` / `Node::inproc` / `Node::grpc` 分别走对应传输。

### UDP 发现（填地址，不选传输）

Broker 默认周期广播 protobuf `BrokerAnnounce`（`proto/robot_bus_interface/msg/v1/announce.proto`）。`decode` 失败或消息内 `magic != "RBUS"` 视为无效丢弃。与 ROS 2 DDS 发现端口段（`7400 + 250 * domainId`）互不冲突。

传输方式仍由你指定；发现只填充位置：

```rust
use robot_bus::{DiscoverOpts, Node, NodeOptions};

// tcp / ipc / inproc / grpc 均可；discover 只填 host / 路径 / URL
let opts = NodeOptions::tcp().discover(DiscoverOpts {
    domain_id: 0,
    broker_id: None, // 同 domain 多于一个 broker 时必填
    ..Default::default()
})?;
let mut node = Node::with_options("talker", opts);
```

或分两步：`discovery::wait(opts)?` → `ann.apply(NodeOptions::grpc())?`。

**同进程 inproc：** ZeroMQ 的 `inproc://` 是 context-local。嵌入式 broker 与 Node 必须共用同一个 [`Context`](../src/runtime/context.rs)：

```rust
use robot_bus::{Context, Node, RobotBusBroker, RobotBusConfig};

let ctx = Context::new();
let broker = RobotBusBroker::start_with_context(ctx.clone(), RobotBusConfig::default())?;
let mut node = Node::inproc_with_context(ctx, "pilot");
// …
broker.stop()?;
```

`RobotBusBroker::start(config)` 仍可用（内部自建 Context）；跨进程 tcp/ipc 不要求共享。

跨 broker（federation）：在 `RobotBusConfig` 的 message/service/action 上设置 `broker_id` 与 `peers`（`MessagePeer` / `ServicePeer` / `ActionPeer`），或 CLI `--broker-id` / `--message-peer` / `--service-peer` / `--action-peer`。各语言嵌入式 start API 使用同款字符串约定（见对应 `*-api.md`）。

**进程内嵌入**（不必单独起二进制）：

```rust
use robot_bus::{RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig::default())?;
// 默认端点同上；需要改端口 / 地址时再填 RobotBusConfig 字段
broker.stop()?;
```

典型流程：`Node::new` → `create_*` → `node.spin()`。多节点或需并行时再 `executor.add_node` + `executor.spin`。仅连 gRPC 网关时用 `Node::grpc` / `Node::grpc_at`（见下文「gRPC 模式 Node」）。

---

## 本地参数（Node）

ROS 2 风格的本节点参数表（不经总线；无远程参数服务 / CLI `-p`）。标量类型：`bool` / `i64` / `f64` / `String`；须先 `declare` 再 `get` / `set`，`set` 时类型必须与声明一致。

可用 YAML 启动加载（未声明则 declare，已声明则 set）：

- 扁平：`max_speed: 1.5`
- ROS 2 风格：`ros__parameters: { … }`
- 通配：`"/**": { ros__parameters: { … } }`

```rust
use robot_bus::{Node, ParameterValue};

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("pilot");
    node.declare_parameter("max_speed", ParameterValue::Double(1.5))?;
    node.declare_parameter("frame_id", ParameterValue::String("base_link".into()))?;

    if let ParameterValue::Double(v) = node.get_parameter("max_speed")? {
        println!("max_speed={v}");
    }
    node.set_parameter("max_speed", ParameterValue::Double(2.0))?;
    assert!(node.has_parameter("frame_id"));
    for p in node.list_parameters() {
        println!("{} = {:?}", p.name, p.value);
    }

    node.load_parameters_from_yaml_str(
        r#"
ros__parameters:
  max_speed: 3.0
  enabled: true
"#,
    )?;
    node.load_parameters_from_yaml_file("config/pilot.yaml")?;
    Ok(())
}
```

---

## Message bus（Node + spin）

接近 ROS 2：`Node::new` → typed `create_publisher` / `create_subscription`（创建时绑定消息类型，自动 encode/decode）→ `node.spin()`。底层与 gRPC 仍传 opaque bytes。

Typed `create_publisher::<M>` 会向 broker 控制面（service bus 服务 `/robot_bus/topic_type/register`）**best-effort** 登记 `topic → M::full_name()`（如 `sensor_msgs.msg.v1.Imu`）。登记失败只打日志，不影响 publish。`create_publisher_raw` 不登记。可用 `rbus topic list` / `rbus topic info /path` 查看（HTTP 默认 `http://127.0.0.1:15770`）。

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("pilot");
    // 进程内 / 自定义地址：Node::with_options("pilot", NodeOptions { ... })

    let imu_pub = node.create_publisher::<Imu>("/robot1/imu")?;
    node.create_subscription::<Imu, _>(
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
    imu_pub.publish(&imu)?;

    node.create_timer(
        Duration::from_millis(100),
        Arc::new(|| {
            // 周期任务
        }),
        None,
    )?;

    let handle = node.shutdown_handle()?;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        handle.shutdown();
    });

    node.spin()?; // 阻塞
    Ok(())
}
```

Raw bytes：`create_publisher_raw` / `create_subscription_raw`。

topic / service / action 名按传入原样使用（请自行写全路径）。

显式 Executor（多节点共享 / 多线程）：

```rust
use robot_bus::{MultiThreadedExecutor, Node, SingleThreadedExecutor};

let mut node = Node::new("pilot");
let executor = SingleThreadedExecutor::new();
executor.add_node(&mut node)?;
// 或 MultiThreadedExecutor::new(4) — 最多 n 个 worker；配合 Reentrant
// callback group 时订阅 / timer / service / action 都可并行。
executor.spin()?;
```

`MutuallyExclusive` 组内仍串行。

### Callback group

接近 ROS 2：`MutuallyExclusive`（组内互斥）与 `Reentrant`（组内可并行，配合 `MultiThreadedExecutor`）。节点默认有一个互斥 group；`callback_group` 传 `None` 时用默认组（与 ROS 2 把 group 作为参数传入一致）。

```rust
use robot_bus::{CallbackGroupType, MultiThreadedExecutor, Node};

let mut node = Node::new("pilot");
let executor = MultiThreadedExecutor::new(4);
executor.add_node(&mut node)?;

let reentrant = node.create_callback_group(CallbackGroupType::Reentrant);
node.create_subscription_raw(
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

与 topic 相同：`Node` → typed `create_service` / `create_client` → `server_node.spin()`。

```rust
use std::time::Duration;
use robot_bus::std_srvs::srv::v1::{SetBool, SetBoolRequest, SetBoolResponse};
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut server_node = Node::new("svc_server");
    let mut cli_node = Node::new("svc_client");

    server_node.create_service::<SetBool, _>(
        "/set_bool",
        |req: SetBoolRequest| SetBoolResponse {
            success: true,
            message: format!("set:{}", req.data),
        },
        None,
    )?;

    let client = cli_node.create_client::<SetBool>("/set_bool")?;
    let handle = server_node.shutdown_handle()?;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        let resp = client
            .call(&SetBoolRequest { data: true }, Some(Duration::from_secs(5)))
            .expect("call");
        assert!(resp.success);
        handle.shutdown();
    });

    server_node.spin()?;
    Ok(())
}
```

Raw bytes：`create_service_raw` / `create_client_raw`。endpoint 取自 `NodeOptions`（`service_frontend` / `service_backend`）。

---

## Action bus

同样挂在 Node 上：typed `create_action_server` / `create_action_client` → `server_node.spin()`。客户端采用 ROS 2 风格 `GoalHandle`：`send_goal` 立即返回，feedback 到达时实时调用回调，result 由 handle 独立等待。

> 下例为概念性 API；GoalHandle API 正在实现中，最终签名以 crate 文档为准。

```rust
use std::time::Duration;
use robot_bus::action::v1::{Fibonacci, FibonacciGoal};
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("act_client");
    let client = node.create_action_client::<Fibonacci>("fibonacci")?;
    let goal = client.send_goal(
        &FibonacciGoal { order: 5 },
        |feedback| println!("feedback: {:?}", feedback.sequence),
    )?; // GoalHandle 立即返回

    // 可从其它控制路径调用；这是 best-effort 请求，不代表服务端已确认。
    // goal.cancel()?;
    let result = goal.result(Some(Duration::from_secs(10)))?;
    assert_eq!(result.sequence, vec![0, 1, 1, 2, 3]);
    Ok(())
}
```

Raw bytes：`create_action_server_raw` / `create_action_client_raw`。ZMQ 的 `cancel()` 发送显式 `CANCEL` 帧；gRPC 的 `cancel()` 取消对应的 server stream，两者都不提供“服务端已确认取消”的保证。

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
    let mut node = Node::with_options("demo", options);
    let imu_pub = node.create_publisher_raw("/robot1/imu")?;
    node.create_subscription_raw(
        "/robot1/imu",
        Arc::new(|topic, payload| println!("{topic}: {} bytes", payload.len())),
        None,
    )?;
    imu_pub.publish(b"hello")?;
    node.spin_once(None)?;

    broker.stop()?;
    Ok(())
}
```

---

## Protobuf 消息（`robot_bus::<pkg>`）

总线与 gRPC 网关仍传 opaque bytes（gRPC 侧通常拿不到业务 proto，保持二进制）。Node SDK 在 create 时绑定类型并自动 encode/decode，例如 `create_publisher::<Imu>` / `create_subscription::<Imu, _>`。消息类型挂在 crate 命名空间下：`robot_bus::sensor_msgs::msg::v1::Imu`。其它消息同理，例如：

类型名约定为 protobuf 全名（`prost::Name::full_name()`，如 `sensor_msgs.msg.v1.Imu`），经 console 控制面登记，**不**写入每条消息帧。

```rust
use robot_bus::geometry_msgs::msg::v1::{Twist, Vector3};

let twist = Twist {
    linear: Some(Vector3 { x: 1.0, y: 0.0, z: 0.0 }),
    angular: Some(Vector3::default()),
};
let pub_ = node.create_publisher::<Twist>("cmd_vel")?;
pub_.publish(&twist)?;
```

Service / Action 同理（如 `create_client::<SetBool>`、`create_action_client::<Fibonacci>`）。

---

## gRPC 模式 Node（客户端）

`Node::grpc` / `NodeOptions::grpc` 通过 broker 的 gRPC 网关接入总线，**不创建 ZMQ socket**。API 仍是 `create_subscription` / `create_publisher` / `create_client` / `create_action_client` + `spin`，对调用方透明。

| 支持 | 不支持 |
|------|--------|
| `create_subscription` / `_raw` | `create_service` / `_raw` |
| `create_publisher` / `_raw` | `create_action_server` / `_raw` |
| `create_client` / `_raw` | 挂到 ZMQ `SingleThreadedExecutor` |
| `create_action_client` / `_raw` | |
| `create_timer`、`spin` / `shutdown` | |

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::Node;

let mut node = Node::grpc("web-client");
// 或 Node::grpc_at("web-client", "http://127.0.0.1:15770");

let pub_ = node.create_publisher_raw("/robot1/cmd")?;
pub_.publish(b"go")?;

node.create_subscription_raw(
    "/robot1/imu",
    Arc::new(|topic, payload| {
        println!("{topic}: {} bytes", payload.len());
    }),
    None,
)?;

let client = node.create_client_raw("svc.echo")?;
let reply = client.call(b"ping", Some(Duration::from_secs(2)))?;

let action = node.create_action_client_raw("act.navigate")?;
let goal = action.send_goal(b"goal", |feedback| {
    println!("feedback: {} bytes", feedback.len());
})?;
let result = goal.result(Some(Duration::from_secs(10)))?;

// 订阅与 action feedback 回调需要 spin；result 独立等待
node.spin()?;
```

上面的 GoalHandle 写法是概念性 API。底层仍是网关 RPC（`MessageGateway.Subscribe` / `MessageGateway.Publish` / `ServiceGateway.Call` / `ActionGateway.SendGoal`）。需要更底层控制时，可直接用下一节的 tonic 客户端。

---

## gRPC / gRPC-Web 网关

由 `RobotBusBroker` / `robot_bus_broker` 一并启动（feature `grpc`，默认开启）。标准 gRPC 与 gRPC-Web **同端口**（默认 `0.0.0.0:15770`）。

| RPC | 说明 |
|-----|------|
| `MessageGateway.Subscribe` | server stream：topic 前缀 → `TopicMessage` |
| `MessageGateway.Publish` | 一元：`TopicMessage` → 写入 message bus XSUB |
| `ServiceGateway.Call` | 一元：`service_name` + request bytes → response bytes |
| `ActionGateway.SendGoal` | 一元 `GoalRequest` → server stream `ActionEvent`（实时 `FEEDBACK`，最终 `RESULT`） |

gRPC action 的取消就是取消该 goal 的响应流，不另发 cancel RPC，也不表示服务端已确认；ZMQ transport 则由 GoalHandle 发显式 `CANCEL` 帧。

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

Publish 示例：

```rust
use robot_bus::grpc::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::grpc::pb::TopicMessage;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessageGatewayClient::connect("http://127.0.0.1:15770").await?;
    client
        .publish(Request::new(TopicMessage {
            topic: "imu".into(),
            payload: b"hello".to_vec(),
        }))
        .await?;
    Ok(())
}
```

Service / Action 示例：

```rust
use robot_bus::grpc::pb::action_gateway_client::ActionGatewayClient;
use robot_bus::grpc::pb::service_gateway_client::ServiceGatewayClient;
use robot_bus::grpc::pb::{ActionKind, GoalRequest, ServiceCallRequest};
use tonic::Request;
use tokio_stream::StreamExt;

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
    let mut stream = act
        .send_goal(Request::new(GoalRequest {
            action_name: "act.demo".into(),
            goal: b"go".to_vec(),
            goal_id: String::new(),
            timeout_ms: 10_000,
        }))
        .await?
        .into_inner();
    while let Some(ev) = stream.next().await {
        let ev = ev?;
        println!("{:?}: {} bytes", ActionKind::try_from(ev.kind), ev.body.len());
    }
    Ok(())
}
```

Proto 包名：`robot_bus_interface.grpc.v1`。见 `proto/robot_bus_interface/grpc/v1/{message,service,action}_gateway.proto`。

UDP 发现：`robot_bus_interface.msg.v1`，见 `proto/robot_bus_interface/msg/v1/announce.proto`。

---

## 传输与端点

一般由 `NodeOptions` 推导；需要手工拼地址时：

```rust
use robot_bus::transports::{
    message_xsub_endpoint, message_xpub_endpoint,
    service_frontend_endpoint, service_backend_endpoint,
    action_frontend_endpoint, action_backend_endpoint,
};

// transport: "tcp" | "ipc" | "inproc"（gRPC 模式用 Node::grpc，不经这些端点）
let ep = message_xpub_endpoint("localhost", "tcp")?;
```

---

## 错误类型

```rust
use robot_bus::{BusError, Result};

match result {
    Err(BusError::Timeout(_)) => { /* client poll 超时；service REQ 会自动重建 socket，可直接再 call */ }
    Err(BusError::NoWorker { name }) => { /* 无 worker / pending 排队超时 */ }
    Err(BusError::WorkerDied { name }) => { /* 飞中 worker/peer 挂掉，broker 合成错误 */ }
    Err(BusError::Cancelled { name }) => { /* action：pending 上的 goal 被 CANCEL */ }
    Err(BusError::NoGoal { goal_id }) => { /* action：未知 goal / 重复 goal_id */ }
    Err(e) => eprintln!("{e}"),
    Ok(v) => { /* ... */ }
}
```

**可靠性语义（本期）：**

- Service / action **不**做 broker 重启续传；调用方重试须使用新的 `request_id` / `goal_id`。
- Service `call` 超时后 socket 已复位，同一 client 可继续调用。
- Action `send_goal` 立即返回 GoalHandle；feedback 回调与 `result()` 等待彼此独立。
- Action `cancel()` / result 超时后的清理均为 best-effort：gRPC 取消响应流，ZMQ 发显式 `CANCEL` 帧；不保证服务端确认。
- Topic pub/sub 仍是 best-effort（无 ACK）。

## 工具节点：Image encoder

主 crate feature **`image-encoder`（默认开启）**：模块 `robot_bus::image_encoder`，二进制 `rbus_image_encoder`。订阅 `sensor_msgs/Image`，发布 `foxglove_msgs/CompressedVideo`（需系统 FFmpeg）。

```bash
brew install ffmpeg   # 或 apt 安装 ffmpeg + libav*-dev
cargo install robot-bus --bin rbus_image_encoder

rbus_image_encoder --print-example-config > encoder.yaml
rbus_image_encoder --params encoder.yaml
```

## 工具节点：Image decoder

主 crate feature **`image-decoder`（默认开启）**：模块 `robot_bus::image_decoder`，二进制 `rbus_image_decoder`。订阅 `foxglove_msgs/CompressedVideo`（H.264/H.265 Annex-B），发布 `sensor_msgs/Image`（需系统 FFmpeg）。

```bash
cargo install robot-bus --bin rbus_image_decoder

rbus_image_decoder --print-example-config > decoder.yaml
rbus_image_decoder --params decoder.yaml
```

## 工具节点：Audio capture / play

主 crate features **`audio-capture` / `audio-play`（默认开启）**：`rbus_audio_capture`、`rbus_audio_play`。

```bash
# Debian/Ubuntu 可能需要：sudo apt install libasound2-dev
cargo install robot-bus --bin rbus_audio_capture
cargo install robot-bus --bin rbus_audio_play

rbus_audio_capture --print-example-config > capture.yaml
rbus_audio_play --print-example-config > play.yaml
```

## 工具节点：USB camera

主 crate feature **`usb-camera`（默认开启）**：模块 `robot_bus::usb_camera`，二进制 `rbus_usb_camera`。经 nokhwa 采集 USB / 摄像头，发布 `sensor_msgs/Image`（`rgb8`），默认话题 `/camera/image_raw`。

```bash
cargo install robot-bus --bin rbus_usb_camera
rbus_usb_camera --list-devices
rbus_usb_camera --print-example-config > camera.yaml
rbus_usb_camera --params camera.yaml
```

## 工具节点：TF（static + robot_state_publisher）

库模块 **`robot_bus::tf`**（始终可用）：`Buffer`、`TfListener`、`TransformBroadcaster`。消息真相源为 `tf2_msgs/TFMessage`（`/tf`、`/tf_static`）。

- feature **`static-transform-publisher`** → `rbus_static_transform_publisher`
- feature **`robot-state-publisher`** → `rbus_robot_state_publisher`（URDF 子集含 `<mimic>` + JointState）

```bash
cargo install robot-bus --bin rbus_static_transform_publisher
cargo install robot-bus --bin rbus_robot_state_publisher
rbus_static_transform_publisher --print-example-config > static_tf.yaml
rbus_robot_state_publisher --print-example-config > rsp.yaml
```

```rust
use robot_bus::tf::TfListener;

let listener = TfListener::with_defaults(&mut node)?;
let buf = listener.buffer();
// after spin delivers /tf + /tf_static:
let t = buf.lock().unwrap().lookup_transform("base_link", "camera", None)?;
```

不要默认多媒体依赖时：`cargo build --no-default-features --features grpc,console`。
