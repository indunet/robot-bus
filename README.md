English | [中文](README-zh.md)

# *Robot Bus*

[![CI](https://github.com/indunet/robot-bus/actions/workflows/ci.yml/badge.svg)](https://github.com/indunet/robot-bus/actions/workflows/ci.yml)
[![Code Quality](https://img.shields.io/github/actions/workflow/status/indunet/robot-bus/dynamic%2Fgithub-code-scanning%2Fcodeql?label=Code%20Quality)](https://github.com/indunet/robot-bus/security/code-scanning)
[![crates.io](https://img.shields.io/crates/v/robot-bus.svg?color=f74c00)](https://crates.io/crates/robot-bus)
[![PyPI](https://img.shields.io/pypi/v/robot-bus.svg?color=3775a9)](https://pypi.org/project/robot-bus/)
[![npm](https://img.shields.io/npm/v/robot-bus.svg?color=cb3837)](https://www.npmjs.com/package/robot-bus)
[![Maven Central](https://img.shields.io/maven-central/v/org.indunet/robot-bus.svg?label=Maven%20Central&color=007396)](https://central.sonatype.com/artifact/org.indunet/robot-bus)
[![License](https://img.shields.io/badge/License-Apache_2.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

Lightweight ROS 2–style messaging over ZeroMQ — topics, services & actions, no ROS install. SDKs for Rust, Python, TypeScript, C++, Java, and Android.

No ROS distro, no `source setup.bash`, no workspace. One broker process plus an SDK in any supported language is enough.

**Design principles**: APIs stay close to ROS 2 naming and usage (`Node`, `SingleThreadedExecutor` / `MultiThreadedExecutor`, `add_node`, `create_publisher` / `create_subscription`, `spin`) to ease migration; the transport is ZeroMQ and is not tied to any ROS distribution.

> **Pre-release notice**: This project is still in pre-release. APIs may change substantially and runtime stability is not production-ready yet — use caution in production.

More API examples live under [`docs/`](docs/).

### Crate API

| Module | Role |
|------|------|
| `broker::` | Routing process (message / service / action) |
| Top-level API | Publisher / Subscriber / Client / Worker |
| `runtime::Executor` | Low-level poll loop (usually wrapped by the executors below) |
| `runtime::SingleThreadedExecutor` / `MultiThreadedExecutor` | Explicit executors (multi-node / parallel); a single node can `Node::spin` directly |
| `runtime::Node` / `TopicPublisher` / `CallbackGroup` | Nodes, publishers, callback groups (mutually exclusive / reentrant) |
| `grpc::` (default feature) | gRPC / gRPC-Web gateway (started with the broker) |
| `ros2::` (`ros2` feature) | In-process ROS 2 topic bridge (`Ros2Bridge`) |

### Repository layout

Rust core stays at the repo root (`Cargo.toml` + `src/`). Language SDKs live under `bindings/`; do not flatten them to peer top-level folders.

| Path | Role |
|------|------|
| [`src/`](src/), `Cargo.toml` | Rust core (crates.io / maturin entry) |
| [`proto/`](proto/) | Contract source: ROS-style Protobuf → generated code for Rust / bindings |
| [`bindings/`](bindings/) | Language SDKs (Python, TypeScript, C++, Java, Android) |
| [`console/`](console/) | Web monitoring console (product UI; build output synced to `assets/console/` locally / in CI, not committed) |
| [`benches/`](benches/) | Perf harnesses: [`robot_bus_perf/`](benches/robot_bus_perf/) (`just perf`), [`ros2_perf/`](benches/ros2_perf/) (`just perf-ros2`) |
| [`tests/`](tests/) | Rust integration tests + cross-language interop (`just test-interop`) |
| [`docs/`](docs/) | API guides and generated perf reports |
| [`scripts/`](scripts/), [`tools/`](tools/), `justfile` | Codegen, packaging, and task orchestration |

## Architecture

```
Application code (Rust / Python / TypeScript / C++ / Java / Android)
  └── robot-bus SDK
              │
              │ ZMQ (tcp / ipc / inproc) or gRPC / gRPC-Web
              ▼
robot_bus_broker process
```

### Optional ROS 2 bridge (Rust feature)

Everyday robot-bus development **does not install ROS 2**. To interconnect with a ROS 2 graph in-process, enable Cargo feature **`ros2`** and use `robot_bus::ros2::Ros2Bridge` (chained API or YAML). Requires a sourced ROS 2 Humble (or later) + `rclrs` link environment. See the [ROS 2 bridge](#ros-2-bridge-feature-ros2) section.

## Quick start

### 1. Start the broker

Rust:

```bash
cargo run --bin robot_bus_broker
# discovery / domain: robot_bus_broker --domain-id 0 --advertise-host 10.0.0.5
# disable announce:     robot_bus_broker --no-discovery
```

### Introspection CLI (`rbus`)

Query the broker console HTTP API (default `http://127.0.0.1:15771`; override with `--url` or `ROBOT_BUS_BROKER_URL`):

```bash
cargo run --bin rbus -- topic list
cargo run --bin rbus -- service list
cargo run --bin rbus -- action list
cargo run --bin rbus -- status
```

Topic list shows names with recent forwarded traffic (a live subscriber is required for metrics). Services / actions appear after a worker READY.

### Broker discovery (UDP multicast)

Brokers periodically announce on `239.255.76.67:15550` (away from ROS 2 / DDS `7400` / `239.255.0.1`). The UDP payload is a pure protobuf [`BrokerAnnounce`](proto/robot_bus_interface/msg/v1/announce.proto) (`magic` must be `RBUS`). Invalid packets are dropped.

Clients still **choose the transport** (`tcp` / `ipc` / `inproc` / `grpc`); discovery only fills host / paths / gRPC URL:

```rust
use robot_bus::{DiscoverOpts, Node, NodeOptions};

let opts = NodeOptions::tcp().discover(DiscoverOpts {
    domain_id: 0,
    ..Default::default()
})?;
let mut node = Node::with_options("talker", opts);
```

Same API shape in bindings: `Node.discover(...)` (Python / C++ / Java / Android / TypeScript Node.js). Browser gRPC-Web has no UDP discovery.

Python (ships a CLI entry after `pip install robot-bus`):

```bash
robot-bus-broker
```

Or start in-process:

```python
import robot_bus

with robot_bus.RobotBusBroker.start() as broker:
    # ... application code ...
    pass
# leaves the with-block and stops automatically

# Or block like the CLI (Ctrl+C to exit)
# robot_bus.run_broker()
```

### Python

```bash
pip install robot-bus
```

Local development (requires [maturin](https://www.maturin.rs/); [just](https://github.com/casey/just) optional):

```bash
just python-dev
# equivalent: cd bindings/python && maturin develop --features extension-module,grpc
```

(`grpc` is a default feature; spelling it out avoids missing the gateway when `default-features = false`.)

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, imu: Imu):
    print(topic, imu.linear_acceleration)

node = robot_bus.Node("pilot")

imu_pub = node.create_publisher("/robot1/imu", Imu)
node.create_subscription("/robot1/imu", on_imu, msg_type=Imu)
imu_pub.publish(Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8)))
# node.spin()  # blocks; call node.shutdown() / shutdown_handle().shutdown() from another thread
```

(Omit the message type for raw bytes. Use `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node` when sharing nodes or needing multi-threaded handlers.)

gRPC-only gateway clients: `Node.grpc("name")` / `Node.grpc_at("name", "http://…")` (subscribe / publish / call service / action). See [`docs/python-api.md`](docs/python-api.md).

### TypeScript

```bash
npm install robot-bus
```

Local development:

```bash
just ts-dev
# equivalent: cd bindings/typescript && npm install && npm run build:native && npm run build:ts
```

One npm package: Node.js uses napi-rs (full ZMQ API); browsers use gRPC-Web (subscribe / publish / service / action client). Bundlers pick the entry via `exports`. See [`docs/typescript-api.md`](docs/typescript-api.md).

```ts
import { Node } from "robot-bus";
import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";

const node = new Node("pilot");
const pub = node.createPublisher("/robot1/imu", Imu);
node.createSubscription("/robot1/imu", (_t, imu) => console.log(imu), Imu);
```

Browser / gRPC-only: `Node.grpc("client")` (the browser entry's `Node` is the gRPC-Web facade).

### Java / Android (Maven Central)

| Artifact | Directory | Coordinates |
|------|------|------|
| JVM JAR (Java 11+, Maven) | [`bindings/java/`](bindings/java/) | `org.indunet:robot-bus` |
| Android AAR (minSdk 24, Kotlin SDK) | [`bindings/android/`](bindings/android/) | `org.indunet:robot-bus-android` |

Package name is `org.indunet.robot.bus` for both. Android is a **standalone** Kotlin SDK (does not depend on the Java JAR). After you write release notes and Publish on GitHub, CI publishes to Maven Central (or run the Actions workflows manually).

```bash
just java-dev       # JVM
just android-dev    # AAR (needs Android SDK + NDK 26 + cargo-ndk)
```

```kotlin
// Android (Kotlin)
RobotBusAndroid.init(this)
val pub = node.createPublisher("/imu", Imu::class.java)
```

See [`docs/java-api.md`](docs/java-api.md) / [`docs/android-api.md`](docs/android-api.md), [`bindings/java/README.md`](bindings/java/README.md) / [`bindings/android/README.md`](bindings/android/README.md).

### C++ (DEB / MSI)

No central package registry for C++: download `robot-bus-cpp_*_linux_*.deb` / `robot-bus-cpp_*_windows_*.msi` / `robot-bus-cpp_*_macos_*.pkg` from [GitHub Releases](https://github.com/indunet/robot-bus/releases) (CI attaches them after you Publish a release). See [`docs/cpp-api.md`](docs/cpp-api.md).

```cpp
#include <robot_bus/Node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

robot_bus::Broker broker;
robot_bus::Node node("pilot");
auto pub = node.create_publisher("/imu");
```

### Rust (Node + spin)

Add to `Cargo.toml`:

```toml
robot-bus = { path = "../robot-bus" }
# or from crates.io: robot-bus = "0.1.0"
```

Semantics mirror ROS 2: `Node::new` → typed `create_publisher` / `create_subscription` → `node.spin()` (auto-attaches a `SingleThreadedExecutor`).

gRPC-only (no ZMQ): `Node::grpc` / `Node::grpc_at` — subscribe, publish, and call service / action, but cannot act as a server; see [`docs/rust-api.md`](docs/rust-api.md#grpc-模式-node客户端).

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::Node;

let mut node = Node::new("pilot");

let imu_pub = node.create_publisher::<Imu>("/robot1/imu")?;
node.create_subscription::<Imu, _>(
    "/robot1/imu",
    |topic, imu| {
        println!("{topic}: {:?}", imu.linear_acceleration);
    },
    None,
)?;

let imu = Imu {
    linear_acceleration: Some(Vector3 { x: 0.0, y: 0.0, z: 9.8 }),
    ..Default::default()
};
imu_pub.publish(&imu)?;

node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| {
        // control period / heartbeat
    }),
    None,
)?;

let handle = node.shutdown_handle()?;
std::thread::spawn(move || { /* ... */ handle.shutdown(); });
node.spin()?;
```

- Single-node default: `node.spin()` (internal `SingleThreadedExecutor`)
- `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node`: shared multi-node or parallel handlers
- Callback groups: `MutuallyExclusive` / `Reentrant` (`create_callback_group`; default is mutually exclusive)
- Service / action: typed `create_service` / `create_client`, `create_action_server` / `create_action_client` (on the Node like topics; `*_raw` variants also available)
- Timer: `create_timer` (also on the Node, driven by `spin`)
- Raw bytes: `create_publisher_raw` / `create_subscription_raw`
- Low-level escape hatch: `Executor` (advanced)

Send / receive high-water marks (ZMQ HWM, not full QoS) can be set at create time or at runtime:

```rust
use robot_bus::{Publisher, HighWaterMark};

let pub_ = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
pub_.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

Defaults: message `STREAM(2/2)`, service `RPC(4/4)`, action `ACTION(8/8)`. Broker flags: `--snd-hwm` / `--rcv-hwm`.

## Binaries

| Binary | Description |
|------|------|
| `robot_bus_broker` | Starts all three buses plus the gRPC / gRPC-Web gateway |

## Web console (`console/`)

Optional monitoring UI for broker status, topic traffic, and event logs. With the `console` feature (default), the broker serves an **embedded** static UI on `0.0.0.0:15771` after you build assets once.

**Development (hot reload — preferred):**

```bash
# terminal 1
cargo run --bin robot_bus_broker
# terminal 2
cd console && pnpm install && pnpm dev
# open http://localhost:3000  (/api is proxied to the broker; override with ROBOT_BUS_BROKER_URL)
```

**Embedded in the broker binary:**

```bash
just console          # pnpm build + sync → assets/console/ (gitignored)
cargo run --bin robot_bus_broker
# open http://localhost:15771
# disable: cargo run --bin robot_bus_broker -- --no-console
```

`assets/console/` is **build output** (not committed). CI and release jobs run `just console` (or equivalent) before compiling with the `console` feature.

Wired to the broker's same-port monitoring API: `GET /api/v1/status`, `GET /api/v1/topics`, `GET /api/v1/services`, `GET /api/v1/actions`, `SSE /api/v1/events`. The frontend source lives in `console/`; only the generated static files are compiled into binaries with the `console` feature.

## gRPC / gRPC-Web gateway

Started with `robot_bus_broker` / `RobotBusBroker::start`. Standard gRPC and gRPC-Web share the **same port** (default `0.0.0.0:15770`).

You can also attach via the Node API with `Node::grpc` / `Node::grpc_at` (client: subscribe / publish / call service / call action; see [`docs/rust-api.md`](docs/rust-api.md#grpc-模式-node客户端)).

| RPC | Semantics |
|-----|------|
| `MessageGateway.Subscribe` | Subscribe by topic prefix; server streams binary payloads |
| `MessageGateway.Publish` | Unary publish: topic + binary payload onto the message bus |
| `ServiceGateway.Call` | Unary: `service_name` + request bytes → response bytes |
| `ActionGateway.Run` | Bidirectional stream: client sends GOAL / CANCEL; server pushes `ActionEvent` (`kind` distinguishes FEEDBACK / RESULT) |

```bash
cargo run --bin robot_bus_broker
# config: cargo run --bin robot_bus_broker -- --help
# gRPC: http://0.0.0.0:15770
```

In-process:

```rust
use robot_bus::{GrpcBrokerConfig, RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig {
    grpc: GrpcBrokerConfig {
        listen: "0.0.0.0:15770".parse()?,
        ..Default::default()
    },
    ..RobotBusConfig::default()
})?;
let grpc = format!("http://{}", broker.grpc_listen());
```

Proto (package `robot_bus_interface.grpc.v1`, distinct from ROS `*.msg.v1` / `*.srv.v1`):

- [`message_gateway.proto`](proto/robot_bus_interface/grpc/v1/message_gateway.proto)
- [`service_gateway.proto`](proto/robot_bus_interface/grpc/v1/service_gateway.proto)
- [`action_gateway.proto`](proto/robot_bus_interface/grpc/v1/action_gateway.proto)

UDP discovery (`robot_bus_interface.msg.v1`):

- [`announce.proto`](proto/robot_bus_interface/msg/v1/announce.proto)

## ROS 2 bridge (`feature = "ros2"`)

In-process topic bridge via `robot_bus::ros2::Ros2Bridge` (chained API or YAML). **Not** enabled by default — core SDK, crates.io, and maturin builds stay ROS-free.

| Need | Notes |
|------|--------|
| Cargo | `--features ros2` (pulls optional `rclrs`) |
| Environment | Source ROS 2 Humble+ so `rcl` / type support libs link; main CI does **not** enable this feature |
| Broker | Running `robot_bus_broker` reachable over tcp/ipc (or `bus_discover`) |
| MVP types | `std_msgs/msg/String`, `sensor_msgs/msg/Imu` (dynamic ROS messages ↔ robot-bus protobuf) |

```rust
use robot_bus::ros2::{Direction, Ros2Bridge};

let mut bridge = Ros2Bridge::new("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
        .string()
        .direction(Direction::Both)
        .add()
    .route("/imu", "/imu")
        .imu()
        .direction(Direction::RosToBus)
        .add()
    .build()?;
bridge.spin()?;
// or: Ros2Bridge::from_yaml("bridge.yaml")?.spin()?;
```

## Testing

```bash
just test-rust
just test-python
just test-typescript
just test-interop   # cross-language matrix under tests/interop/
just perf           # robot-bus → docs/perf-report.md (benches/robot_bus_perf/)
just perf-ros2      # ROS 2 comparison under benches/ros2_perf/
# equivalent:
# cargo test
# PYTHONPATH=bindings/python python3 bindings/python/tests/test_msgs_roundtrip.py
# PYTHONPATH=bindings/python python3 bindings/python/tests/test_typed_api.py
# cd bindings/typescript && npm test
```

## Protobuf messages

[`proto/`](proto/) follows ROS package layout: `proto/<pkg>/{msg|srv|grpc}/v1/*.proto`.

Generated stubs are **not checked into git**; run `just gen-*` after changing protos or before local tests (requires protoc **35.1**). CI / release pipelines generate and ship them inside wheels, crates.io crates, npm packages, DEB/MSI, and Maven JAR/AAR — **consumers of published packages do not need protoc**.

| Language | Path | Notes |
|------|------|------|
| Rust | `robot_bus::<pkg>::{msg\|srv}::v1` | `just gen-rust` → `src/generated/<pkg>/{msg\|srv}/v1/<stem>.rs` |
| Python | `robot_bus.<pkg>.{msg\|srv}.v1` | `just gen-python`; packed into the wheel |
| TypeScript | `robot-bus/<pkg>/{msg\|srv}/v1/…` | `just gen-typescript`; packed into the npm package |
| Java / Android | `org.indunet.robot.bus.<pkg>.{msg\|srv\|action}.v1` | `just gen-java`; packed into JAR / AAR |
| C++ | `#include <robot_bus/…>` | `just gen-cpp`; packed into DEB/MSI |

- Transport body remains opaque bytes (including the gRPC gateway); the Rust Node SDK binds types at create time and auto encode/decode (`create_publisher::<M>`, etc.), or use `*_raw`; Python / TypeScript / **Java** pass a protobuf type for typed APIs (thin wrappers), or omit the type for raw bytes
- **srv** is a pair of `*Request` / `*Response` messages, not gRPC
- **grpc** (`robot_bus`) is the gateway RPC contract, started with the broker (default feature `grpc`)
- Messages live under the `robot_bus` namespace and do **not** claim top-level ROS package names like `sensor_msgs`; encoding is protobuf and is not interoperable with ROS CDR
- One-shot: `just gen-all`

Covered packages: `builtin_interfaces`, `std_msgs`, `std_srvs`, `geometry_msgs`, `sensor_msgs`, `nav_msgs`, `tf2_msgs`, `trajectory_msgs`, `diagnostic_msgs`, `unique_identifier_msgs`, `shape_msgs`, `visualization_msgs`, `control_msgs`, `nav2_msgs`, `foxglove_msgs` (ported from [Foxglove schemas](https://github.com/foxglove/foxglove-sdk), package `foxglove_msgs.msg.v1`).
