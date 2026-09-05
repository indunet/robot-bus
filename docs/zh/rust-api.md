[English](../en/rust-api.md) | 中文

# Rust API示例

`Cargo.toml`：

```toml
robot-bus = "2.3.0"
# 本地：robot-bus = { path = "../robot-bus" }
# 默认已启用 WebSocket RPC网关（`ws` feature）；若需关闭：robot-bus = { version = "2.3.0", default-features = false }
```

## Broker启动

**更鼓励在程序里启动 broker**（`RobotBusBroker::start`），让 broker 与业务同进程。CLI 适合演示、多进程示例，或单独拉起常驻 broker。默认一次启动 **message / service / action** 三条总线；每条 TCP默认 bind `…:0`（由操作系统分配空闲端口），并启动 **API**（WebSocket RPC `/ws` / `GET /api/v1/discover` / console http），默认 `0.0.0.0:15570`。

```rust
use robot_bus::{RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig::default())?;
// 默认端点见下表；需要改端口 / 地址时再填 RobotBusConfig字段
broker.stop()?;
```

需要独立进程时再用 CLI：

```bash
cargo run --bin robot_bus_broker
# 查看参数：cargo run --bin robot_bus_broker -- --help
# 只要 TCP：加 --tcp-only
# API口：--api-listen 0.0.0.0:15570（别名 --console-listen）
# 对外可达主机：--advertise-host HOST
# 邦联：--peer 10.0.0.2:15570（对端 API口；可重复）
# 控制台侧栏：--no-tank / --no-docs（文档默认显示）
```

**默认端点：**

| 角色 | tcp | ipc | inproc |
|------|-----|-----|--------|
| message XSUB / XPUB | `tcp://0.0.0.0:0`（启动后解析为实际端口） | `ipc:///tmp/robot_bus/<broker_id>/…` | `inproc://robot_bus/…` |
| service FE / BE | `tcp://0.0.0.0:0` | 同上 | 同上 |
| action FE / BE | `tcp://0.0.0.0:0` | 同上 | 同上 |
| API（WebSocket RPC + discover + console） | `0.0.0.0:15570` | — | — |

仍可用 `--message-xsub-bind`等手动固定总线端口。SDK侧**推荐** `Context::new` → `Node::with_context`（本地 tcp）；便捷的 `Node::new`仍可用（私有 Context）。端点未填时自动对 `http://127.0.0.1:15570`做 discover；`Node::ipc` / `Node::inproc` / `Node::ws`分别走对应传输。

### HTTP发现（填地址，不选传输）

对已知 API口请求 `GET /api/v1/discover`，拿到可连接的 ZMQ端点。传输方式仍由你指定；发现只填充位置：

```rust
use robot_bus::{DiscoverOpts, Node, NodeOptions};

let opts = NodeOptions::tcp().discover(DiscoverOpts {
    api_url: "http://127.0.0.1:15570".into(),
    broker_id: None, // 可选过滤
    ..Default::default()
})?;
let mut node = Node::with_options("talker", opts);
```

或分两步：`discovery::wait(opts)?` → `ann.apply(NodeOptions::ws())?`。

UDP组播发现已移除。

`Node::new` / `Node::tcp` **不会**阻塞等 broker。broker未启动时构造也不失败；会话在后台重试 HTTP discover。用 [`ConnectionState`](../../src/runtime/session.rs) 和 `wait_for_broker`：

```rust
use std::time::Duration;
use robot_bus::Node;

let mut node = Node::new("pilot");
if !node.wait_for_broker(Some(Duration::from_secs(5))) {
    anyhow::bail!("broker not reachable (state={})", node.connection_state());
}
node.add_on_connection_event(|old, new, reason| {
    eprintln!("{old} -> {new} ({reason})");
});
```

`spin` / `start`在 broker重启后会继续重试。`create_*`会短等 discover，仍未连上则返回带当前 state的错误。WebSocket节点共用这套会话合同；Connected表示 `/ws`套接字已连通。

**同进程 inproc：** ZeroMQ的 `inproc://`是 context-local。嵌入式 broker与 Node必须共用同一个 [`Context`](../../src/runtime/context.rs)：

```rust
use robot_bus::{Context, Node, RobotBusBroker, RobotBusConfig};

let ctx = Context::new();
let broker = RobotBusBroker::start_with_context(&ctx, RobotBusConfig::default())?;
let mut node = Node::inproc_with_context(&ctx, "pilot");
// …
broker.stop()?;
```

`RobotBusBroker::start(config)`仍可用（内部自建 Context）；跨进程 tcp/ipc不要求共享。

跨 broker（federation）：优先 `--peer HOST:PORT`（对端 API口，内部会 `GET /api/v1/discover`填齐 ZMQ peers），或在 `RobotBusConfig`上设置 `broker_id`与 `peers`（`MessagePeer` / `ServicePeer` / `ActionPeer`），或 CLI `--broker-id` / `--message-peer` / `--service-peer` / `--action-peer`。各语言嵌入式 start API使用同款字符串约定（见对应 `*-api.md`）。Message federation **不会**转发保留命名空间 `/robot_bus`（含 `/robot_bus/status`、topology、bot等 console系统 topic），避免多 broker帮连时状态快照互相覆盖；用户业务 topic仍按需推送。

典型流程：`Context` → `Node::with_context` → `create_*` → `node.spin()`（或便捷 `Node::new`）。多节点或需并行时再 `executor.add_node` + `executor.spin`。仅连 WebSocket RPC网关时用 `Node::ws` / `Node::ws_at`（见下文「WebSocket RPC模式 Node」）。

---

## 本地参数（Node）

ROS2风格的本节点参数表（不经总线；无远程参数服务 / CLI `-p`）。标量类型：`bool` / `i64` / `f64` / `String`。

对齐 ROS2的用法：`declare_parameter` / `get_parameter`返回 [`Parameter`]（含 `name` + `value`），`set_parameter(Parameter::new(...))`，以及 `get_parameters` / `set_parameters` / `list_parameters(prefixes, depth)` / `undeclare_parameter`。可用 `as_bool` / `as_int` / `as_double` / `as_string`取值。

YAML启动加载（未声明则 declare，已声明则 set）：

- 扁平：`max_speed: 1.5`
- ROS2风格：`ros__parameters: { … }`
- 通配：`"/**": { ros__parameters: { … } }`

```rust
use robot_bus::{Context, Node, Parameter};

fn main() -> robot_bus::Result<()> {
    let ctx = Context::new();
    let mut node = Node::with_context(&ctx, "pilot");
    node.declare_parameter("max_speed", 1.5)?;
    node.declare_parameter("frame_id", "base_link")?;

    let max_speed = node.get_parameter("max_speed")?.as_double()?;
    println!("max_speed={max_speed}");
    node.set_parameter(Parameter::new("max_speed", 2.0))?;
    assert!(node.has_parameter("frame_id"));

    let listed = node.list_parameters(&[], 0); // depth 0 = recursive
    println!("names={:?}", listed.names);

    for p in node.list_all_parameters() {
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

接近 ROS2：`Context` → `Node::with_context` → typed `create_publisher` / `create_subscription`（创建时绑定消息类型，自动 encode/decode）→ `node.spin()`。底层与 gRPC仍传 opaque bytes。

Typed `create_publisher::<M>`会向 broker控制面（service bus服务 `/robot_bus/topic_type/register`）**best-effort** 登记 `topic → M::full_name()`（如 `sensor_msgs.msg.v1.Imu`）。登记失败只打日志，不影响 publish。`create_publisher_raw`不登记。可用 `rbus topic list` / `rbus topic info /path`查看（HTTP默认 `http://127.0.0.1:15570`）。

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::{Context, Node};

fn main() -> robot_bus::Result<()> {
    let ctx = Context::new();
    let mut node = Node::with_context(&ctx, "pilot");
    // 进程内 / 自定义地址：Node::with_context_options(&ctx, "pilot", NodeOptions { ... })

    let imu_pub = node.create_publisher::<Imu>("/robot1/imu")?;
    let _sub = node.create_subscription::<Imu, _>(
        "/robot1/imu",
        |imu| {
            println!("angular_z={:?}", imu.angular_velocity);
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

    let _timer = node.create_wall_timer(
        Duration::from_millis(100),
        Arc::new(|| {
            // 周期任务（create_timer别名）
        }),
        None,
    )?;

    // destroy_subscription / destroy_service / destroy_action_server
    // 与 cancel_timer相同：executor start() 活跃时拒绝。
    // wait_for_broker / connection_state / add_on_connection_event
    // wait_for_message / client.wait_for_service / wait_for_action_server可用。
    // create_*_with_qos(..., QosProfile::keep_last(n), ...) 设置 KeepLast depth。

    let handle = node.shutdown_handle()?;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        handle.shutdown();
    });

    node.spin()?; // 阻塞
    Ok(())
}
```

完整可运行程序：[`examples/topic_imu/`](../../examples/topic_imu/)。

Raw bytes：`create_publisher_raw` / `create_subscription_raw`。

topic / service / action名按传入原样使用（请自行写全路径）。

显式 Executor（多节点共享 / 多线程）：

```rust
use robot_bus::{MultiThreadedExecutor, Node, SingleThreadedExecutor};

let mut node = Node::new("pilot");
let executor = SingleThreadedExecutor::new();
executor.add_node(&mut node)?;
// 或 MultiThreadedExecutor::new(4) — n条常驻 worker从队列取任务；
// 配合 Reentrant callback group时订阅 / timer / service / action可并行。
// 有 worker池时回调不会在 poll线程上跑。
executor.spin()?;
```

`MutuallyExclusive`组内仍串行（同时最多一个在跑，只占一条 worker）。

### Callback group

接近 ROS2：`MutuallyExclusive`（组内互斥，同时最多一个回调，多余的在组内排队）与 `Reentrant`（组内可并行，配合 `MultiThreadedExecutor`的常驻线程池）。节点默认有一个互斥 group；`callback_group`传 `None`时用默认组（与 ROS2把 group作为参数传入一致）。`SingleThreadedExecutor`上两种 group都在 poll线程串行执行。

```rust
use robot_bus::{CallbackGroupType, MultiThreadedExecutor, Node};

let mut node = Node::new("pilot");
let executor = MultiThreadedExecutor::new(4);
executor.add_node(&mut node)?;

let reentrant = node.create_callback_group(CallbackGroupType::Reentrant);
node.create_subscription_raw(
    "/robot1/imu",
    Arc::new(|_| { /* 可与同组其它回调并行 */ }),
    Some(&reentrant),
)?;
node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| {}),
    Some(&reentrant),
)?;
```


### 高水位（HWM）与 QoS

`QosProfile::keep_last(depth)`映射为 ZMQ HWM。Topic用 PUB/SUB HWM；service / action用 DEALER HWM（`snd` / `rcv`都等于 depth）。reliability固定 best-effort（RPC也没有 DDS reliability）。不传 QoS则用节点默认（topic 8/8，service 4/4，action 8/8）。

**WebSocket** Node上 KeepLast **只兑现订阅侧**：作为网关到客户端的队列深度（满则丢；不传则用网关默认 64）。发布侧 QoS忽略（所有 WS发布者共用网关的一个 PUB）。WS的 service / action客户端没有 ZMQ socket，HWM被忽略。

```rust
use robot_bus::{Node, QosProfile, Publisher, HighWaterMark};

let mut node = Node::new("pilot");
let pub_ = node.create_publisher_with_qos::<robot_bus::sensor_msgs::msg::v1::Imu>(
    "/robot1/imu",
    QosProfile::keep_last(10),
)?;
node.create_subscription_with_qos::<robot_bus::sensor_msgs::msg::v1::Imu, _>(
    "/robot1/imu",
    QosProfile::keep_last(10),
    |_imu| {},
    None,
)?;

// Service / action：同样 KeepLast → DEALER HWM
// node.create_service_with_qos::<SetBool, _>("/reset", QosProfile::keep_last(16), handler, None)?;
// let client = node.create_client_with_qos::<SetBool>("/reset", QosProfile::keep_last(16))?;
// node.create_action_server_with_qos::<Fibonacci, _>("/fib", QosProfile::keep_last(16), handler, None)?;
// let act = node.create_action_client_with_qos::<Fibonacci>("/fib", QosProfile::keep_last(16))?;

// 底层仍可直接设 HWM：
let raw = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
raw.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

---

## Service bus

与 topic相同：`Node` → typed `create_service` / `create_client` → `server_node.spin()`。

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

Raw bytes：`create_service_raw` / `create_client_raw`。KeepLast：`create_service_with_qos` / `create_client_with_qos`（以及 `*_raw_with_qos`）。endpoint取自 `NodeOptions`（`service_frontend` / `service_backend`）。

---

## Action bus

同样挂在 Node上：typed `create_action_server` / `create_action_client` → `server_node.spin()`。客户端采用 ROS2风格 `GoalHandle`：`send_goal`立即返回，feedback到达时实时调用回调，result由 handle独立等待。

```rust
use std::time::Duration;
use robot_bus::example_interfaces::action::v1::{Fibonacci, FibonacciGoal};
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("act_client");
    let client = node.create_action_client::<Fibonacci>("fibonacci")?;
    let goal = client.send_goal(
        &FibonacciGoal { order: 5 },
        |feedback| println!("feedback: {:?}", feedback.sequence),
    )?; // GoalHandle立即返回

    // 可从其它控制路径调用；这是 best-effort请求，不代表服务端已确认。
    // goal.cancel()?;
    let result = goal.result(Some(Duration::from_secs(10)))?;
    assert_eq!(result.sequence, vec![0, 1, 1, 2, 3]);
    Ok(())
}
```

完整可运行程序：[`examples/service_set_bool/`](../../examples/service_set_bool/)、[`examples/action_fibonacci/`](../../examples/action_fibonacci/)。

Raw bytes：`create_action_server_raw` / `create_action_client_raw`。KeepLast：`create_action_server_with_qos` / `create_action_client_with_qos`（以及 `*_raw_with_qos`）。ZMQ的 `cancel()`发送显式 `CANCEL`帧；gRPC的 `cancel()`取消对应的 server stream，两者都不提供“服务端已确认取消”的保证。

---

## 进程内 broker

不必单独起 `robot_bus_broker`二进制；用 `NodeOptions`填入 broker返回的 bind地址：

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
        Arc::new(|payload| println!("{} bytes", payload.len())),
        None,
    )?;
    imu_pub.publish(b"hello")?;
    node.spin_once(None)?;

    broker.stop()?;
    Ok(())
}
```

---

## Protobuf消息（`robot_bus::<pkg>`）

总线与 gRPC网关仍传 opaque bytes（gRPC侧通常拿不到业务 proto，保持二进制）。Node SDK在 create时绑定类型并自动 encode/decode，例如 `create_publisher::<Imu>` / `create_subscription::<Imu, _>`。消息类型挂在 crate命名空间下：`robot_bus::sensor_msgs::msg::v1::Imu`。其它消息同理，例如：

类型名约定为 protobuf全名（`prost::Name::full_name()`，如 `sensor_msgs.msg.v1.Imu`），经 console控制面登记，**不**写入每条消息帧。

```rust
use robot_bus::geometry_msgs::msg::v1::{Twist, Vector3};

let twist = Twist {
    linear: Some(Vector3 { x: 1.0, y: 0.0, z: 0.0 }),
    angular: Some(Vector3::default()),
};
let pub_ = node.create_publisher::<Twist>("cmd_vel")?;
pub_.publish(&twist)?;
```

Service / Action同理（如 `create_client::<SetBool>`、`create_action_client::<Fibonacci>`）。

---

## WebSocket RPC模式 Node（客户端）

`Node::ws` / `NodeOptions::ws`通过 broker的 WebSocket RPC网关接入总线，**不创建 ZMQ socket**。API仍是 `create_subscription` / `create_publisher` / `create_client` / `create_action_client` + `spin`，对调用方透明。

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

let mut node = Node::ws("web-client");
// 或 Node::ws_at("web-client", "http://127.0.0.1:15570");

let pub_ = node.create_publisher_raw("/robot1/cmd")?;
pub_.publish(b"go")?;

node.create_subscription_raw(
    "/robot1/imu",
    Arc::new(|payload| {
        println!("{} bytes", payload.len());
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

// 订阅与 action feedback回调需要 spin；result独立等待
node.spin()?;
```

底层是多路复用 WebSocket RPC（V3 opcode：Subscribe / Publish / Call / SendGoal）。请用 `Node::ws`，不要直接拼帧。

---

## WebSocket RPC网关

由 `RobotBusBroker` / `robot_bus_broker`一并启动（feature `ws`，默认开启）。原生与浏览器客户端共用 API端口上的 **`/ws`**（默认 `0.0.0.0:15570`）。**不兼容旧版：** V3成帧；不再接受 V2的 method字符串和 `TopicMessage`信封。

| Opcode | RPC | REQUEST头 | DATA payload |
|--------|-----|------------|--------------|
| 1 | Subscribe | topic前缀 + `qos_depth` | `u16 topic_len` + topic + 原始总线字节 |
| 2 | Publish | topic；body = 原始总线字节 | 无（成功只回 TRAILER） |
| 3 | Call | 服务名 + timeout + request id；body = 原始请求 | 原始响应 |
| 4 | SendGoal | action名 + goal id + timeout；body = 原始 goal | `u8 kind` + 原始 body（`FEEDBACK`后接 `RESULT`） |

`CANCEL` / `TRAILER` / `PING` / `PONG`含义不变（`stream_id`；客户端用奇数 id）。Action cancel：WebSocket发显式 `CANCEL`帧并继续等到 `RESULT`，断连仍会提交 cancel。ZMQ transport由 GoalHandle发显式 `CANCEL`帧。均不表示服务端已确认。

HTTP发现：`GET /api/v1/discover`（JSON）；历史 protobuf `BrokerAnnounce`仅作兼容编码辅助，UDP组播路径已移除。

---

## 传输与端点

一般由 `NodeOptions`推导；需要手工拼地址时：

```rust
use robot_bus::transports::{
    message_xsub_endpoint, message_xpub_endpoint,
    service_frontend_endpoint, service_backend_endpoint,
    action_frontend_endpoint, action_backend_endpoint,
};

// transport: "tcp" | "ipc" | "inproc"（WebSocket RPC模式用 Node::ws，不经这些端点）
let ep = message_xpub_endpoint("localhost", "tcp")?;
```

---

## 错误类型

```rust
use robot_bus::{BusError, Result};

match result {
    Err(BusError::Timeout(_)) => { /* client poll超时；service REQ会自动重建 socket，可直接再 call */ }
    Err(BusError::NoWorker { name }) => { /* 无 worker / pending排队超时 */ }
    Err(BusError::WorkerDied { name }) => { /* 飞中 worker/peer挂掉，broker合成错误 */ }
    Err(BusError::Cancelled { name }) => { /* action：pending上的 goal被 CANCEL */ }
    Err(BusError::NoGoal { goal_id }) => { /* action：未知 goal / 重复 goal_id */ }
    Err(e) => eprintln!("{e}"),
    Ok(v) => { /* ... */ }
}
```

**可靠性语义（本期）：**

- Service / action **不**做 broker重启续传；调用方重试须使用新的 `request_id` / `goal_id`。
- Service `call`超时后 socket已复位，同一 client可继续调用。
- Action `send_goal`立即返回 GoalHandle；feedback回调与 `result()`等待彼此独立。
- Action `cancel()` / result超时后的清理均为 best-effort：WebSocket发显式 CANCEL帧（断连仍会 cancel）；WebSocket RPC取消响应流；ZMQ发显式 `CANCEL`帧；不保证服务端确认。
- Topic pub/sub仍是 best-effort（无 ACK）。
