[English](../en/rust-api.md) | 中文

# Rust API

`Cargo.toml`：

```toml
robot-bus = "3.3.0"
# 本地：robot-bus = { path = "../crates/robot-bus" }
# 一个包同时包含 SDK 和 broker。默认的 `full` feature 会打开 broker、
# WebSocket（客户端和服务端）、内嵌控制台和 tank 演示。
# 只写客户端时可以全部关闭：robot-bus = { version = "3.3.0", default-features = false }
```

## Broker启动

**更鼓励在程序里启动 broker**（`RobotBusBroker::start`），让 broker 与业务同进程。CLI 适合演示、多进程示例，或单独拉起常驻 broker。默认一次启动 **message / service / action** 三条总线；每条 TCP默认 bind `…:0`（由操作系统分配空闲端口），并启动 **API**（WebSocket RPC `/ws-rpc` / `GET /api/v1/discover` / console http），默认 `0.0.0.0:15560`。

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
# API口：--api-listen 0.0.0.0:15560（别名 --console-listen）
# TCP 数据主机与 API 入口不同时：--advertise-host HOST
# 邦联：--peer 10.0.0.2:15560（对端 API口；可重复）
# 控制台侧栏：--no-tank / --no-docs（文档默认显示）
```

**默认端点：**

| 角色 | tcp | ipc | inproc |
|------|-----|-----|--------|
| message XSUB / XPUB | `tcp://0.0.0.0:0`（启动后解析为实际端口） | broker 专用临时目录（见发现结果 `ipcDir`） | `inproc://robot_bus/…` |
| service FE / BE | `tcp://0.0.0.0:0` | 同上 | 同上 |
| action FE / BE | `tcp://0.0.0.0:0` | 同上 | 同上 |
| API（WebSocket RPC + discover + console） | `0.0.0.0:15560` | — | — |

Broker 在系统临时目录下创建独立的 IPC 子目录，关闭时清理。需要指定可写父目录时，使用 `RobotBusBroker::start_with_context_in`。IPC 需要 ZeroMQ 构建和操作系统同时支持；同机客户端应从发现结果取实际路径，而不是拼接目录名。

仍可用 `--message-xsub-bind`等手动固定总线端口。SDK侧**推荐** `Context::new` → `Node::with_context`（本地 tcp）；便捷的 `Node::new`仍可用（私有 Context）。端点未填时自动对 `http://127.0.0.1:15560`做 discover；`Node::ipc` / `Node::inproc` / `Node::ws`分别走对应传输。

### HTTP发现（填地址，不选传输）

对已知 API 口请求 `GET /api/v1/discover`，拿到 ZMQ 数据端口。默认情况下，原生 TCP 客户端和 `--peer` 用发现 URL 的主机名或 IP 加这些端口来连接。TCP 数据主机与 API 入口不同时，用 `--advertise-host HOST` 覆盖。WebSocket 发现则沿用 HTTP API 入口。传输方式仍由你指定；发现只填充位置：

发现 IPC 端点时用 `NodeOptions::for_discovery("ipc")`，不要用预填了固定路径的 `NodeOptions::ipc()`。

如果直接读取 `DiscoverResponse`，调用 `response.to_announcement(api_base)` 时传入实际访问的 API 地址，即可解析数据端点。

```rust
use robot_bus::{DiscoverOpts, Node, NodeOptions};

let opts = NodeOptions::tcp().discover(DiscoverOpts {
    api_url: "http://127.0.0.1:15560".into(),
    broker_id: None, // 可选过滤
    ..Default::default()
})?;
let mut node = Node::with_options("talker", opts);
```

WebSocket 也可分两步发现：`discovery::wait(opts)?` → `ann.apply(NodeOptions::for_discovery("ws"))?`。

UDP组播发现已移除。

`Node::new` / `Node::tcp` **不会**阻塞等 broker。broker未启动时构造也不失败；会话在后台重试 HTTP discover。用 [`ConnectionState`](../../crates/robot-bus/src/runtime/session.rs) 和 `wait_for_broker`：

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

`spin` / `start`在 broker重启后会继续重试。`create_*`会短等 discover，仍未连上则返回带当前 state的错误。WebSocket节点共用这套会话合同；Connected表示 `/ws-rpc`套接字已连通。

**同进程 inproc：** ZeroMQ的 `inproc://`是 context-local。嵌入式 broker与 Node必须共用同一个 [`Context`](../../crates/robot-bus/src/runtime/context.rs)：

```rust
use robot_bus::{Context, Node, RobotBusBroker, RobotBusConfig};

let ctx = Context::new();
let broker = RobotBusBroker::start_with_context(&ctx, RobotBusConfig::default())?;
let mut node = Node::inproc_with_context(&ctx, "pilot");
// …
broker.stop()?;
```

`RobotBusBroker::start(config)`仍可用（内部自建 Context）；跨进程 tcp/ipc不要求共享。

配置 [Broker 联邦](architecture.md#broker-联邦)时，先启动目标 Broker，再给发起加入的一方传入 `--peer HOST:PORT`，端口填目标的 HTTP API 端口。目标在发现响应中声明 `federation: ["join"]` 时，只需在一侧配置，话题、服务和 Action 即可双向转发。跨主机用 TCP，同一主机可用 `--peer ipc+HOST:PORT`，通过 HTTP 发现，再连接目标的 `ipc://` 套接字。

对接缺少 `join` 能力的旧版时，两侧都要配置对端。可以通过 `RobotBusConfig` 设置 `broker_id` 和 `peers`（`MessagePeer` / `ServicePeer` / `ActionPeer`），也可以使用 `--broker-id`、`--message-peer`、`--service-peer` 和 `--action-peer`。显式配置 IPC 端点时，为每个 Broker 用 `--ipc-dir DIR`（`DiscoveryConfig::ipc_dir`）固定独立目录，便于两侧引用 `ipc://DIR/<通道>.ipc`。嵌入式启动 API 使用同样的 peer 字符串。保留的 `/robot_bus` 话题只在本地传递。

典型流程：`Context` → `Node::with_context` → `create_*` → `node.spin()`（或便捷 `Node::new`）。多节点或需并行时再 `executor.add_node` + `executor.spin`。仅连 WebSocket RPC服务端时用 `Node::ws` / `Node::ws_at`（见下文「WebSocket RPC模式 Node」）。

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

Typed `create_publisher::<M>`会向 broker控制面（service bus服务 `/robot_bus/topic_type/register`）**best-effort** 登记 `topic → M::full_name()`（如 `sensor_msgs.msg.v1.Imu`）。登记失败只打日志，不影响 publish。`create_publisher_raw`不登记。可用 `rbus topic list` / `rbus topic info /path`查看（HTTP默认 `http://127.0.0.1:15560`）。

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


### 回调异常、过载与停止

启用 Rust panic 展开时，回调 panic 会被记录并隔离。订阅或定时器异常不会导致工作线程退出或回调组停止调度。服务和动作处理函数异常时，原生总线返回 `BusError::HandlerPanicked`（`HANDLER_PANICKED\0name`），WebSocket 服务端返回 Internal 状态。

常驻线程池在等待任务达到 1024 个时停止接受新任务，并为每个工作线程保留一个内部续执行名额，避免繁忙回调组长期占用线程；每个互斥回调组也默认最多等待 1024 个回调，可用 `CallbackGroup::with_queue_capacity(kind, capacity)` 调整组容量。队列满时立即拒绝新任务，不阻塞消息接收线程：订阅和定时器回调被丢弃并记录日志；服务和动作请求返回 `BusError::Busy`（`BUSY\0name`，WebSocket 为 ResourceExhausted）。已接受的同组回调按先后顺序执行。这些执行器限制独立于传输层 HWM，不提供“覆盖为最新值”的策略。

`CallbackGroup::queue_stats()` 提供组内等待数和拒绝数，`Executor::worker_queue_stats()` 提供常驻线程池队列统计。`executor.shutdown()` 和 `executor.shutdown_handle()` 不需要获取 spin 循环的锁，可在其他线程运行 spin 时调用。停止信号用于退出轮询，不会强制终止已经执行的用户回调。

### 高水位（HWM）与 QoS

`QosProfile::keep_last(N)` 将每个原生 Node 订阅的待处理队列限制为最近 N 条。队列满时先丢最旧的一条，再放入新样本。收包线程独立于用户回调，慢回调运行期间也会继续收包。`keep_last(1)` 只保留最新的待处理样本。创建时省略深度，则使用当前接收 HWM 作为初始深度（通常为 8）。前缀订阅匹配的数据共享该订阅的一条队列。

用 `set_stream_hwm` 单独设置原生 SUB 的传输队列。发布和原生 Service、Action 的 QoS 深度仍用于各自的套接字 HWM。已开始执行的回调继续运行，ZeroMQ 和网络缓冲仍有自己的容量，Topic 投递为尽力而为。低层 `Subscriber` 使用套接字 HWM 直接收包。

WebSocket 订阅的 KeepLast 限制服务端待发送队列，也限制 Rust WS Node 等待回调的队列。省略深度或 `depth ≤ 0` 时取 64，最大为 1,048,576。同一过滤器的回调共享队列并使用同一有效深度。WS 发布和 RPC 忽略深度。新版 WS 客户端通过 opcode 5 请求丢旧，旧 Broker 需要先升级。

`GET /api/v1/subscriptions` 和控制台 Topics 页显示 WS 订阅积压及丢弃数。API 的 `dequeued` 统计服务端从队列取出的次数。`totalDropped` 包括已关闭订阅，Broker 重启时归零。此统计范围为 WS 服务端队列。见[按需构建](build-profiles.md)。

#### 状态话题与 `transient_local`

在原生发布和订阅两端配置 `QosProfile::keep_last(N).transient_local()`。发布者在其生命周期内保留最后一条样本，即使发布时没有订阅者。晚加入或重连的订阅者可以收到这个样本，之后继续接收更新。两端使用完整的 Topic 名称进行精确匹配。普通订阅仍可接收启用状态保留的发布者后续发布的样本。重连时可再次收到同一份状态。

首版每个发布者保留一份内存样本，独立于订阅的 KeepLast 深度。适合设备配置、地图元数据和低频定位状态。发布者关闭后该样本随之释放。原生 TCP、IPC 和 inproc 节点支持该选项，WebSocket 节点目前使用普通实时订阅。

```rust
use robot_bus::{Node, QosProfile, Publisher, HighWaterMark};

let mut node = Node::new("pilot");
let state_qos = QosProfile::keep_last(1).transient_local();
let publisher = node.create_publisher_raw_with_qos("/robot1/map_state", state_qos)?;
publisher.publish(b"map-ready")?;
node.create_subscription_raw_with_qos(
    "/robot1/map_state", state_qos, std::sync::Arc::new(|_state| {}), None,
)?;

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

总线与 WebSocket RPC 服务端传递 opaque bytes，不解析业务消息结构。Node SDK在 create时绑定类型并自动 encode/decode，例如 `create_publisher::<Imu>` / `create_subscription::<Imu, _>`。消息类型挂在 crate命名空间下：`robot_bus::sensor_msgs::msg::v1::Imu`。其它消息同理，例如：

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

`Node::ws` / `NodeOptions::ws`通过 broker的 WebSocket RPC服务端接入总线，**不创建 ZMQ socket**。API仍是 `create_subscription` / `create_publisher` / `create_client` / `create_action_client` + `spin`，对调用方透明。

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
// 或 Node::ws_at("web-client", "http://127.0.0.1:15560");

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

## WebSocket RPC服务端

Rust WebSocket 服务端模块为 `robot_bus::ws`。Broker 监听配置使用 `robot_bus::WsConfig`；独立服务端配置使用 `robot_bus::ws::WsServerConfig`。请求处理类型为 `WsMessageService`、`WsServiceHandler` 和 `WsActionHandler`。更新已有 Rust 集成时，需将模块导入和类型引用同步改为这些名称。`/ws-rpc` 地址、帧格式、`ws` feature 和客户端 API 保持不变。

由 `RobotBusBroker` / `robot_bus_broker`一并启动（feature `ws`，默认开启）。原生与浏览器客户端共用 API端口上的 **`/ws-rpc`**（默认 `0.0.0.0:15560`）。**不兼容旧版：** V3成帧；不再接受 V2的 method字符串和 `TopicMessage`信封。

| Opcode | RPC | REQUEST头 | DATA payload |
|--------|-----|------------|--------------|
| 1 | Subscribe | topic前缀 + `qos_depth` | `u16 topic_len` + topic + 原始总线字节 |
| 5 | SubscribeWithPolicy | topic前缀 + `qos_depth` + overflow策略字节；新版 KeepLast 使用 1（丢旧） | 与 Subscribe 相同 |
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

服务调用的超时共用一个截止时间，覆盖等待共享客户端套接字、发送和接收。服务端调用还包括等待空闲客户端和阻塞工作线程的调度时间。服务端最多提供 8 个活动客户端和 256 个额外调用名额；过载返回 ResourceExhausted。派发前已过期的请求不会发送。已经送达服务端的请求可能在调用方超时后继续执行；超时不意味着撤销服务端操作。

## 错误类型

```rust
use robot_bus::{BusError, Result};

match result {
    Err(BusError::Timeout(_)) => { /* client poll超时；service REQ会自动重建 socket，可直接再 call */ }
    Err(BusError::NoWorker { name }) => { /* 无 worker / pending排队超时 */ }
    Err(BusError::WorkerDied { name }) => { /* 飞中 worker/peer挂掉，broker合成错误 */ }
    Err(BusError::Cancelled { name }) => { /* action：CANCELED */ }
    Err(BusError::ActionAborted(_)) => { /* action：ABORTED */ }
    Err(BusError::ActionRejected(_)) => { /* action：执行前被拒绝 */ }
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
