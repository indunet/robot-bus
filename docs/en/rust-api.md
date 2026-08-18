English | [中文](../zh/rust-api.md)

# Rust API examples

`Cargo.toml`:

```toml
robot-bus = "1.2.2"
# Local: robot-bus = { path = "../robot-bus" }
# WebSocket RPC gateway (`ws` feature) is on by default; to disable: robot-bus = { version = "1.2.2", default-features = false }
```

## Broker startup

Start the broker before running examples (or embed it in-process; see below). By default one startup brings up **message / service / action** buses; each TCP bind defaults to `…:0` (OS assigns a free port), plus **API** (WebSocket RPC `/ws` / `GET /api/v1/discover` / console http), default `0.0.0.0:15570`. `--grpc-listen` is an alias of `--api-listen`.

```bash
cargo run --bin robot_bus_broker
# See options: cargo run --bin robot_bus_broker -- --help
# TCP only: add --tcp-only
# API port: --api-listen 0.0.0.0:15570 (aliases --grpc-listen / --console-listen)
# Externally reachable host: --advertise-host HOST
# Federation: --peer 10.0.0.2:15570 (peer API port; repeatable)
```

**Default endpoints:**

| Role | tcp | ipc | inproc |
|------|-----|-----|--------|
| message XSUB / XPUB | `tcp://0.0.0.0:0` (resolved to actual port after startup) | `ipc:///tmp/robot_bus/<broker_id>/…` | `inproc://robot_bus/…` |
| service FE / BE | `tcp://0.0.0.0:0` | same as above | same as above |
| action FE / BE | `tcp://0.0.0.0:0` | same as above | same as above |
| API (WebSocket RPC + discover + console) | `0.0.0.0:15570` | — | — |

You can still manually pin bus ports with `--message-xsub-bind` and similar flags. On the SDK side, **prefer** `Context::new` → `Node::with_context` (local tcp); convenience `Node::new` still works (private Context). When endpoints are unset it auto-discovers against `http://127.0.0.1:15570`; `Node::ipc` / `Node::inproc` / `Node::ws` use the corresponding transport.

### HTTP discovery (fill addresses, not transport)

Request `GET /api/v1/discover` on a known API port to obtain connectable ZMQ endpoints. You still choose the transport; discovery only fills in locations:

```rust
use robot_bus::{DiscoverOpts, Node, NodeOptions};

let opts = NodeOptions::tcp().discover(DiscoverOpts {
    api_url: "http://127.0.0.1:15570".into(),
    broker_id: None, // optional filter
    ..Default::default()
})?;
let mut node = Node::with_options("talker", opts);
```

Or in two steps: `discovery::wait(opts)?` → `ann.apply(NodeOptions::ws())?`.

UDP multicast discovery has been removed.

**Same-process inproc:** ZeroMQ `inproc://` is context-local. An embedded broker and Node must share the same [`Context`](../../src/runtime/context.rs):

```rust
use robot_bus::{Context, Node, RobotBusBroker, RobotBusConfig};

let ctx = Context::new();
let broker = RobotBusBroker::start_with_context(&ctx, RobotBusConfig::default())?;
let mut node = Node::inproc_with_context(&ctx, "pilot");
// …
broker.stop()?;
```

`RobotBusBroker::start(config)` still works (creates its own Context internally); cross-process tcp/ipc does not require sharing.

Cross-broker (federation): prefer `--peer HOST:PORT` (peer API port; internally `GET /api/v1/discover` fills ZMQ peers), or set `broker_id` and `peers` on `RobotBusConfig` (`MessagePeer` / `ServicePeer` / `ActionPeer`), or CLI `--broker-id` / `--message-peer` / `--service-peer` / `--action-peer`. Embedded start APIs in other languages use the same string conventions (see the corresponding `*-api.md`). Message federation **does not** forward the reserved namespace `/robot_bus` (including `/robot_bus/status`, topology, bot, and other console system topics), avoiding status snapshot overwrites when multiple brokers are bridged; user business topics are still pushed as needed.

**In-process embed** (no separate binary required):

```rust
use robot_bus::{RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig::default())?;
// Default endpoints as above; set RobotBusConfig fields when you need different ports / addresses
broker.stop()?;
```

Typical flow: `Context` → `Node::with_context` → `create_*` → `node.spin()` (or convenience `Node::new`). For multiple nodes or parallelism, use `executor.add_node` + `executor.spin`. For the WebSocket RPC gateway only, use `Node::ws` / `Node::ws_at` (see “WebSocket RPC mode Node” below).

---

## Local parameters (Node)

ROS 2–style local parameter table for this node (not over the bus; no remote parameter service / CLI `-p`). Scalar types: `bool` / `i64` / `f64` / `String`.

Aligned with ROS 2: `declare_parameter` / `get_parameter` return a `Parameter` (`name` + `value`), `set_parameter(Parameter::new(...))`, plus `get_parameters` / `set_parameters` / `list_parameters(prefixes, depth)` / `undeclare_parameter`. Use `as_bool` / `as_int` / `as_double` / `as_string` to extract values.

Load at startup from YAML (undeclared keys are declared; declared keys are set):

- Flat: `max_speed: 1.5`
- ROS 2 style: `ros__parameters: { … }`
- Wildcard: `"/**": { ros__parameters: { … } }`

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

## Message bus (Node + spin)

Close to ROS 2: `Context` → `Node::with_context` → typed `create_publisher` / `create_subscription` (message type bound at creation, auto encode/decode) → `node.spin()`. Under the hood, gRPC still carries opaque bytes.

Typed `create_publisher::<M>` **best-effort** registers `topic → M::full_name()` (e.g. `sensor_msgs.msg.v1.Imu`) with the broker control plane (service bus service `/robot_bus/topic_type/register`). Registration failure is logged only and does not block publish. `create_publisher_raw` does not register. Inspect with `rbus topic list` / `rbus topic info /path` (HTTP default `http://127.0.0.1:15570`).

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::{Context, Node};

fn main() -> robot_bus::Result<()> {
    let ctx = Context::new();
    let mut node = Node::with_context(&ctx, "pilot");
    // In-process / custom address: Node::with_context_options(&ctx, "pilot", NodeOptions { ... })

    let imu_pub = node.create_publisher::<Imu>("/robot1/imu")?;
    let _sub = node.create_subscription::<Imu, _>(
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

    let _timer = node.create_wall_timer(
        Duration::from_millis(100),
        Arc::new(|| {
            // Periodic task (alias of create_timer)
        }),
        None,
    )?;

    // destroy_subscription / destroy_service / destroy_action_server
    // reject while executor start() is active (same as cancel_timer).
    // wait_for_message / client.wait_for_service / wait_for_action_server available.

    let handle = node.shutdown_handle()?;
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        handle.shutdown();
    });

    node.spin()?; // blocking
    Ok(())
}
```

Full runnable programs: [`examples/topic_imu/`](../../examples/topic_imu/).

Raw bytes: `create_publisher_raw` / `create_subscription_raw`.

Topic / service / action names are used as passed (use full paths yourself).

Explicit Executor (shared across nodes / multithreaded):

```rust
use robot_bus::{MultiThreadedExecutor, Node, SingleThreadedExecutor};

let mut node = Node::new("pilot");
let executor = SingleThreadedExecutor::new();
executor.add_node(&mut node)?;
// Or MultiThreadedExecutor::new(4) — up to n workers; with Reentrant
// callback groups, subscriptions / timers / services / actions can run in parallel.
executor.spin()?;
```

Callbacks within a `MutuallyExclusive` group remain serial.

### Callback group

Close to ROS 2: `MutuallyExclusive` (exclusive within the group) and `Reentrant` (parallel within the group, with `MultiThreadedExecutor`). The node has a default mutually exclusive group; when `callback_group` is `None`, the default group is used (same as ROS 2 passing the group as a parameter).

```rust
use robot_bus::{CallbackGroupType, MultiThreadedExecutor, Node};

let mut node = Node::new("pilot");
let executor = MultiThreadedExecutor::new(4);
executor.add_node(&mut node)?;

let reentrant = node.create_callback_group(CallbackGroupType::Reentrant);
node.create_subscription_raw(
    "/robot1/imu",
    Arc::new(|_topic, _payload| { /* may run in parallel with other callbacks in the same group */ }),
    Some(&reentrant),
)?;
node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| {}),
    Some(&reentrant),
)?;
```


### High water mark (HWM) and topic QoS

Topics can use `QosProfile::keep_last(depth)` (→ HWM). **Topics only**; reliability is fixed best-effort. Service / action do not take QoS yet.

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
    |_topic, _imu| {},
    None,
)?;

// Lower-level HWM still available:
let raw = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
raw.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

---

## Service bus

Same as topics: `Node` → typed `create_service` / `create_client` → `server_node.spin()`.

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

Raw bytes: `create_service_raw` / `create_client_raw`. Endpoints come from `NodeOptions` (`service_frontend` / `service_backend`).

---

## Action bus

Also on `Node`: typed `create_action_server` / `create_action_client` → `server_node.spin()`. The client uses a ROS 2–style `GoalHandle`: `send_goal` returns immediately, feedback callbacks run as feedback arrives, and the handle waits for the result independently.

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
    )?; // GoalHandle returns immediately

    // Callable from other control paths; this is a best-effort request, not server confirmation.
    // goal.cancel()?;
    let result = goal.result(Some(Duration::from_secs(10)))?;
    assert_eq!(result.sequence, vec![0, 1, 1, 2, 3]);
    Ok(())
}
```

Full runnable programs: [`examples/service_set_bool/`](../../examples/service_set_bool/), [`examples/action_fibonacci/`](../../examples/action_fibonacci/).

Raw bytes: `create_action_server_raw` / `create_action_client_raw`. ZMQ `cancel()` sends an explicit `CANCEL` frame; gRPC `cancel()` cancels the corresponding server stream—neither guarantees “server acknowledged cancel”.

---

## In-process broker

No need to run the `robot_bus_broker` binary separately; fill bind addresses from the broker into `NodeOptions`:

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

## Protobuf messages (`robot_bus::<pkg>`)

The bus and gRPC gateway still carry opaque bytes (gRPC typically has no business proto—binary is preserved). The Node SDK binds types at create time and auto encode/decodes, e.g. `create_publisher::<Imu>` / `create_subscription::<Imu, _>`. Message types live under the crate namespace: `robot_bus::sensor_msgs::msg::v1::Imu`. Other messages likewise, for example:

Type names follow the protobuf full name (`prost::Name::full_name()`, e.g. `sensor_msgs.msg.v1.Imu`), registered via the console control plane—not written into every message frame.

```rust
use robot_bus::geometry_msgs::msg::v1::{Twist, Vector3};

let twist = Twist {
    linear: Some(Vector3 { x: 1.0, y: 0.0, z: 0.0 }),
    angular: Some(Vector3::default()),
};
let pub_ = node.create_publisher::<Twist>("cmd_vel")?;
pub_.publish(&twist)?;
```

Service / Action likewise (e.g. `create_client::<SetBool>`, `create_action_client::<Fibonacci>`).

---

## WebSocket RPC mode Node (client)

`Node::ws` / `NodeOptions::ws` reach the bus through the broker WebSocket RPC gateway and **do not create ZMQ sockets**. Transport `"grpc"` / older `Node::grpc` names are aliases. The API is still `create_subscription` / `create_publisher` / `create_client` / `create_action_client` + `spin`, transparent to callers.

| Supported | Not supported |
|-----------|---------------|
| `create_subscription` / `_raw` | `create_service` / `_raw` |
| `create_publisher` / `_raw` | `create_action_server` / `_raw` |
| `create_client` / `_raw` | Attach to ZMQ `SingleThreadedExecutor` |
| `create_action_client` / `_raw` | |
| `create_timer`, `spin` / `shutdown` | |

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::Node;

let mut node = Node::ws("web-client");
// Or Node::ws_at("web-client", "http://127.0.0.1:15570");

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

// Subscriptions and action feedback callbacks need spin; result waits independently
node.spin()?;
```

Under the hood this is gateway RPC (`MessageGateway.Subscribe` / `MessageGateway.Publish` / `ServiceGateway.Call` / `ActionGateway.SendGoal`). For lower-level control, use the client in the next section directly.

---

## WebSocket RPC gateway

Started together by `RobotBusBroker` / `robot_bus_broker` (feature `ws`, on by default). **WebSocket RPC** (HTTP/2) and browser **WebSocket RPC** (`/ws`, one RPC per connection) share **the same port** (default `0.0.0.0:15570`). gRPC-Web has been removed.

| RPC | Description |
|-----|-------------|
| `MessageGateway.Subscribe` | Server stream: topic prefix → `TopicMessage` |
| `MessageGateway.Publish` | Unary: `TopicMessage` → write to message bus XSUB |
| `ServiceGateway.Call` | Unary: `service_name` + request bytes → response bytes |
| `ActionGateway.SendGoal` | Unary `GoalRequest` → server stream `ActionEvent` (live `FEEDBACK`, final `RESULT`) |

Action cancel: WebSocket RPC (native and browser) sends an explicit `CANCEL` frame and still waits for `RESULT`; disconnect still submits cancel. ZMQ transport sends an explicit `CANCEL` frame via GoalHandle. None imply server acknowledgment.

Subscribe example:

```rust
use robot_bus::ws_gateway::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::ws_gateway::pb::SubscribeRequest;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessageGatewayClient::connect("http://127.0.0.1:15570").await?;
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

Publish example:

```rust
use robot_bus::ws_gateway::pb::message_gateway_client::MessageGatewayClient;
use robot_bus::ws_gateway::pb::TopicMessage;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessageGatewayClient::connect("http://127.0.0.1:15570").await?;
    client
        .publish(Request::new(TopicMessage {
            topic: "imu".into(),
            payload: b"hello".to_vec(),
        }))
        .await?;
    Ok(())
}
```

Service / Action example:

```rust
use robot_bus::ws_gateway::pb::action_gateway_client::ActionGatewayClient;
use robot_bus::ws_gateway::pb::service_gateway_client::ServiceGatewayClient;
use robot_bus::ws_gateway::pb::{ActionKind, GoalRequest, ServiceCallRequest};
use tonic::Request;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut svc = ServiceGatewayClient::connect("http://127.0.0.1:15570").await?;
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

    let mut act = ActionGatewayClient::connect("http://127.0.0.1:15570").await?;
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

Proto package name: `robot_bus_interfaces.grpc.v1`. See `proto/robot_bus_interfaces/grpc/v1/{message,service,action}_gateway.proto`.

HTTP discovery: `GET /api/v1/discover` (JSON); legacy protobuf `BrokerAnnounce` is encoding compatibility only—UDP multicast path removed.

---

## Transports and endpoints

Usually derived from `NodeOptions`; when assembling addresses by hand:

```rust
use robot_bus::transports::{
    message_xsub_endpoint, message_xpub_endpoint,
    service_frontend_endpoint, service_backend_endpoint,
    action_frontend_endpoint, action_backend_endpoint,
};

// transport: "tcp" | "ipc" | "inproc" (WebSocket RPC mode uses Node::ws, not these endpoints)
let ep = message_xpub_endpoint("localhost", "tcp")?;
```

---

## Error types

```rust
use robot_bus::{BusError, Result};

match result {
    Err(BusError::Timeout(_)) => { /* client poll timeout; service REQ auto-rebuilds socket, call again */ }
    Err(BusError::NoWorker { name }) => { /* no worker / pending queue timeout */ }
    Err(BusError::WorkerDied { name }) => { /* in-flight worker/peer died, broker synthesized error */ }
    Err(BusError::Cancelled { name }) => { /* action: goal on pending was CANCELled */ }
    Err(BusError::NoGoal { goal_id }) => { /* action: unknown goal / duplicate goal_id */ }
    Err(e) => eprintln!("{e}"),
    Ok(v) => { /* ... */ }
}
```

**Reliability semantics (current release):**

- Service / action **do not** survive broker restart; retries must use new `request_id` / `goal_id`.
- After service `call` timeout the socket is reset; the same client can call again.
- Action `send_goal` returns GoalHandle immediately; feedback callbacks and `result()` wait independently.
- Action `cancel()` / cleanup after result timeout are best-effort: WebSocket sends explicit CANCEL frame (disconnect still cancels); WebSocket RPC cancels response stream; ZMQ sends explicit `CANCEL` frame; no server acknowledgment guaranteed.
- Topic pub/sub remains best-effort (no ACK).

## Utility node: Image encoder

Main crate feature **`image-encoder` (default on)**: module `robot_bus::image_encoder`, binary `rbus_image_encoder`. Subscribes to `sensor_msgs/Image`, publishes `foxglove_msgs/CompressedVideo` (requires system FFmpeg).

```bash
brew install ffmpeg   # or apt install ffmpeg + libav*-dev
cargo install robot-bus --bin rbus_image_encoder

rbus_image_encoder --print-example-config > encoder.yaml
rbus_image_encoder --params encoder.yaml
```

## Utility node: Image decoder

Main crate feature **`image-decoder` (default on)**: module `robot_bus::image_decoder`, binary `rbus_image_decoder`. Subscribes to `foxglove_msgs/CompressedVideo` (H.264/H.265 Annex-B), publishes `sensor_msgs/Image` (requires system FFmpeg).

```bash
cargo install robot-bus --bin rbus_image_decoder

rbus_image_decoder --print-example-config > decoder.yaml
rbus_image_decoder --params decoder.yaml
```

## Utility node: Audio capture / play

Main crate features **`audio-capture` / `audio-play` (default on)**: `rbus_audio_capture`, `rbus_audio_play`.

```bash
# Debian/Ubuntu may need: sudo apt install libasound2-dev
cargo install robot-bus --bin rbus_audio_capture
cargo install robot-bus --bin rbus_audio_play

rbus_audio_capture --print-example-config > capture.yaml
rbus_audio_play --print-example-config > play.yaml
```

## Utility node: USB camera

Main crate feature **`usb-camera` (default on)**: module `robot_bus::usb_camera`, binary `rbus_usb_camera`. Captures USB / camera via nokhwa, publishes `sensor_msgs/Image` (`rgb8`), default topic `/camera/image_raw`.

```bash
cargo install robot-bus --bin rbus_usb_camera
rbus_usb_camera --list-devices
rbus_usb_camera --print-example-config > camera.yaml
rbus_usb_camera --params camera.yaml
```

To avoid default multimedia dependencies: `cargo build --no-default-features --features ws,console`.
