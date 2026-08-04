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
| `ros2::` (`ros2` feature) | In-process ROS 2 topic/service bridge (`Ros2Bridge`) |

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

Everyday robot-bus development **does not install ROS 2**. To interconnect with a ROS 2 graph in-process, enable Cargo feature **`ros2`** and use `robot_bus::ros2::Ros2Bridge` (chained API or YAML). Official support: **Humble** and **Jazzy** (source that distro + `rclrs`). C++: install `robot-bus-ros2-humble` or `…-jazzy` (does not vendor `rcl`). See the [ROS 2 bridge](#ros-2-bridge-feature-ros2) section.

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
cargo run --bin rbus -- topic info /robot1/imu
cargo run --bin rbus -- service list
cargo run --bin rbus -- action list
cargo run --bin rbus -- status
```

`topic list` prints `name` and registered protobuf type (or `-`). Types appear after a typed `create_publisher::<M>` registers with the console (before any traffic). Topics with only raw traffic and no registration still list with type `-`. Services / actions appear after a worker READY.

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

No central package registry for C++: download from [GitHub Releases](https://github.com/indunet/robot-bus/releases) (CI attaches assets after you Publish):

| Package | Contents |
|---------|----------|
| `robot-bus_*_linux_*.deb` (also MSI / PKG) | Core SDK + broker, **no** ROS 2 bridge |
| `robot-bus-ros2-humble_*_linux_*.deb` | Same + bridge linked for **Humble** (Linux only; needs system Humble; does not vendor `rcl`) |
| `robot-bus-ros2-jazzy_*_linux_*.deb` | Same + bridge linked for **Jazzy** (Linux only) |

Install only one of the three (they conflict). See [`docs/cpp-api.md`](docs/cpp-api.md).

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
# or from crates.io: robot-bus = "0.1.4"
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

Optional monitoring UI for broker status, topic traffic, event logs, **live topology**, a **Flow** canvas for config-driven plumbing (`rbus_*` + ROS 2 bridge), and a **LIVE** tab for WHEP WebRTC playback from `rbus_webrtc`. With the `console` feature (default), the broker serves an **embedded** static UI on `0.0.0.0:15771` after you build assets once.

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

Wired to the broker's same-port monitoring API: `GET /api/v1/status`, `GET /api/v1/topics`, `GET /api/v1/services`, `GET /api/v1/actions`, `GET /api/v1/topology`, `SSE /api/v1/events`. Topology uses best-effort registration from `Node` create_publisher/subscription. The **Flow** tab edits plumbing nodes and topic wires (export `flow.yaml` / launch commands; does not start processes or hot-apply a running bridge). Legacy Ros2Bridge YAML imports upgrade into a single `ros2_bridge` node. The **LIVE** tab connects directly to a `rbus_webrtc` WHEP URL (default `http://127.0.0.1:8090/whep`; CORS-enabled on the node). The frontend source lives in `console/`; only the generated static files are compiled into binaries with the `console` feature.

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

## Tool nodes (Cargo features)

Tool binaries ship with the main `robot-bus` crate as **default features**. Install system deps (FFmpeg / ALSA headers), then:

```bash
cargo install robot-bus --bin rbus_image_encoder
cargo install robot-bus --bin rbus_image_decoder
cargo install robot-bus --bin rbus_audio_capture
cargo install robot-bus --bin rbus_audio_play
cargo install robot-bus --bin rbus_usb_camera
cargo install robot-bus --bin rbus_apriltag_detector
cargo install robot-bus --bin rbus_xbox_joy
cargo install robot-bus --bin rbus_static_transform_publisher
cargo install robot-bus --bin rbus_robot_state_publisher
```

Skip them with `--no-default-features --features grpc,console` when you only need the library.

### Image encoder (`rbus_image_encoder`)

Subscribes to `sensor_msgs/Image` (`rgb8` / `bgr8` / `mono8`) and publishes `foxglove_msgs/CompressedVideo` (`h264` or `h265`, Annex-B) via **system FFmpeg**. Encoder preference: NVENC → VideoToolbox → `libopenh264` / soft encoders.

```bash
# macOS
brew install ffmpeg
# Debian/Ubuntu
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev \
  libswscale-dev libswresample-dev libavdevice-dev libavfilter-dev

cargo install robot-bus --bin rbus_image_encoder
rbus_image_encoder --print-example-config > encoder.yaml
rbus_image_encoder --params encoder.yaml
```

Linking GPL software encoders (`libx264` / `libx265`) is a deployment choice; prefer hardware encoders when available.

### Image decoder (`rbus_image_decoder`)

Subscribes to `foxglove_msgs/CompressedVideo` (`h264` / `h265`, Annex-B) and publishes `sensor_msgs/Image` (`rgb8` or `bgr8`) via **system FFmpeg**. Decoder preference: NVDEC → VideoToolbox → soft `h264` / `hevc`. Default topics match the encoder: `/camera/video` → `/camera/image_decoded`.

```bash
cargo install robot-bus --bin rbus_image_decoder
rbus_image_decoder --print-example-config > decoder.yaml
rbus_image_decoder --params decoder.yaml
```

### Audio capture (`rbus_audio_capture`)

Captures microphone PCM in **shared** (non-exclusive) mode via [cpal](https://github.com/RustAudio/cpal) and publishes `foxglove_msgs/RawAudio` (`pcm-s16`). Defaults: 16 kHz mono, 20 ms chunks. Feature `audio-capture` (default on).

```bash
# Debian/Ubuntu
sudo apt install libasound2-dev

cargo install robot-bus --bin rbus_audio_capture
rbus_audio_capture --list-devices
rbus_audio_capture --print-example-config > capture.yaml
rbus_audio_capture --params capture.yaml
```

### Audio play (`rbus_audio_play`)

Subscribes to `foxglove_msgs/RawAudio` (`pcm-s16`) and plays on a speaker (cpal shared mode). Feature `audio-play` (default on). Incoming rate/channels must match node parameters.

```bash
cargo install robot-bus --bin rbus_audio_play
rbus_audio_play --list-devices
rbus_audio_play --print-example-config > play.yaml
rbus_audio_play --params play.yaml
```

### USB camera (`rbus_usb_camera`)

Captures USB / webcam frames via [nokhwa](https://github.com/l1npengtul/nokhwa) (V4L2 / AVFoundation / Media Foundation) and publishes `sensor_msgs/Image` (`rgb8`). Defaults: 640×480 @ 30 fps on `/camera/image_raw` — ready for `rbus_image_encoder`. Feature `usb-camera` (default on). On macOS, grant camera permission when prompted.

```bash
cargo install robot-bus --bin rbus_usb_camera
rbus_usb_camera --list-devices
rbus_usb_camera --print-example-config > camera.yaml
rbus_usb_camera --params camera.yaml
```

### AprilTag detector (`rbus_apriltag_detector`)

Subscribes to `sensor_msgs/Image` (`rgb8` / `bgr8` / `mono8`) and publishes `apriltag_msgs/AprilTagDetectionArray` (2D geometry + quality; no pose / TF). Uses the [apriltag](https://crates.io/crates/apriltag) crate (official AprilTag C library, statically linked by default). Defaults: `tag36h11` on `/camera/image_raw` → `/apriltag/detections`. Feature `apriltag-detector` (default on). Needs a C toolchain + CMake to build the bundled AprilTag sources.

```bash
# macOS (Xcode CLT / cmake)
xcode-select --install
brew install cmake
# Debian/Ubuntu
sudo apt install build-essential cmake

cargo install robot-bus --bin rbus_apriltag_detector
rbus_apriltag_detector --print-example-config > apriltag.yaml
# Pipeline: rbus_usb_camera → rbus_apriltag_detector
rbus_apriltag_detector --params apriltag.yaml
```

### WebRTC / WHEP (`rbus_webrtc`)

Subscribes to configurable `sensor_msgs/Image`, `foxglove_msgs/RawAudio` (`pcm-s16`), and optional raw **data** topics, then serves a **WHEP** livestream (H.264 + Opus + DataChannels). Image→H.264 reuses the same FFmpeg `FrameEncoder` as `rbus_image_encoder`. Feature `webrtc` is **off by default** (needs FFmpeg + libopus).

Watch in Console → **LIVE** (preferred), or open the node's demo page at `http://<host>:8090/`.

```bash
# macOS
brew install ffmpeg opus
# Debian/Ubuntu
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev \
  libswscale-dev libswresample-dev libavdevice-dev libavfilter-dev libopus-dev

cargo build --features webrtc --bin rbus_webrtc
# or: cargo install robot-bus --features webrtc --bin rbus_webrtc

rbus_webrtc --print-example-config > webrtc.yaml
# Typical pipeline: rbus_usb_camera (+ optional rbus_audio_capture) → rbus_webrtc
rbus_webrtc --params webrtc.yaml
# Console LIVE → WHEP URL http://127.0.0.1:8090/whep → Connect
```

### Xbox joy (`rbus_xbox_joy`)

Reads a standard USB Xbox-layout pad / wireless receiver via [gilrs](https://gitlab.com/gilrs-project/gilrs) (SDL GameController mappings; typically plug-and-play) and publishes `robot_bus_interface/XboxJoy`. Subscribes to `robot_bus_interface/XboxJoyRumble` for dual-motor vibration. Defaults: `/xbox_joy` out, `/xbox_joy/rumble` in, 50 Hz. Feature `xbox-joy` (default on). Rumble works on Linux / Windows; macOS supports input only.

```bash
# Debian/Ubuntu (gilrs needs libudev)
sudo apt install libudev-dev

cargo install robot-bus --bin rbus_xbox_joy
rbus_xbox_joy --list-devices
rbus_xbox_joy --print-example-config > xbox.yaml
rbus_xbox_joy --params xbox.yaml
```

### Static TF (`rbus_static_transform_publisher`)

Publishes fixed parent→child transforms as `tf2_msgs/TFMessage` on `/tf_static` (ROS TF2 convention). Configure edges in YAML (`translation` + `rotation_rpy` or `rotation_xyzw`). Feature `static-transform-publisher` (default on). Match sensor `frame_id` values (e.g. USB camera `frame_id: camera`) to `child_frame_id` so the tree connects.

```bash
cargo install robot-bus --bin rbus_static_transform_publisher
rbus_static_transform_publisher --print-example-config > static_tf.yaml
rbus_static_transform_publisher --params static_tf.yaml
```

### Robot state publisher (`rbus_robot_state_publisher`)

Loads a URDF (subset: `fixed` / `revolute` / `continuous` / `prismatic`, plus `<mimic>`), subscribes to `sensor_msgs/JointState`, and publishes movable joints on `/tf` plus fixed joints on `/tf_static`. Feature `robot-state-publisher` (default on). Pair with `rbus_ethercat_joint` (or any JointState source); keep joint **names** aligned with the URDF. Mimic joints use `q = multiplier * q_master + offset` and ignore any published value for the mimic joint itself. Drivers should **not** also broadcast TF for the same links.

```bash
cargo install robot-bus --bin rbus_robot_state_publisher
# Point urdf_file at your model (sample: src/robot_state_publisher/examples/simple_arm.urdf)
rbus_robot_state_publisher --print-example-config > rsp.yaml
rbus_robot_state_publisher --params rsp.yaml
```

### TF library (`robot_bus::tf`)

Always available (no feature gate). `Buffer` / `TfListener` subscribe to `/tf` + `/tf_static` and expose `lookup_transform` / `can_transform`. v1 time semantics: static edges always apply; dynamic edges use the **latest** sample (no interpolation). `TransformBroadcaster` helps publish `TFMessage` batches.

Frame naming convention: prefer ROS-style `map` / `odom` / `base_link` / `*_link`. Message `header.frame_id` on sensors should match a link or static child in the tree.

### EtherCAT joints (`rbus_ethercat_joint`)

Independent tool node (same pattern as camera / xbox — **not** part of the broker). Bridges EtherCAT / CiA402 drives: publishes `sensor_msgs/JointState`, subscribes to `robot_bus_interface/JointCommand`. Supports cyclic modes **CSP / CSV / CST** via YAML `mode` per joint. Feature `ethercat-joint` is **off by default** (needs a Linux NIC and usually `CAP_NET_RAW` / root for real hardware).

```bash
cargo install robot-bus --bin rbus_ethercat_joint --features ethercat-joint
rbus_ethercat_joint --print-example-config > ethercat_joint.yaml
# Edit iface, joints, PDO offsets; use backend: mock without hardware
rbus_ethercat_joint --params ethercat_joint.yaml
rbus_ethercat_joint --params ethercat_joint.yaml --list-devices
```

Optional services (same process): `std_srvs/SetBool` on `enable_service` (default `/ethercat_joint/enable`) and `std_srvs/Trigger` on `fault_reset_service` (default `/ethercat_joint/fault_reset`). For secondary development, depend on `robot-bus` with `ethercat-joint` and call `robot_bus::ethercat_joint::run_with_hooks` with a custom `JointHooks` impl.

**Safety:** treat EtherCAT enable as hazardous. Use an external STO / e-stop; this node’s command-timeout and diagnostics are not a certified safety function.

## ROS 2 bridge (`feature = "ros2"`)

In-process topic, service, **and** action bridge via `robot_bus::ros2::Ros2Bridge` (chained API or YAML). **Not** enabled by default — core SDK, crates.io, and maturin builds stay ROS-free.

**Supported ROS 2 distributions (official):** **Humble** and **Jazzy**. Other distros: build from source after sourcing that distro (best-effort).

| Need | Notes |
|------|--------|
| Cargo (Rust) | `--features ros2` (pulls optional `rclrs`) |
| Environment | Source **Humble** or **Jazzy** so `rcl` / type support libs link; main CI does **not** enable this feature |
| C++ packages | `robot-bus` (no bridge) vs `robot-bus-ros2-humble` / `robot-bus-ros2-jazzy` (mutually exclusive, **Linux DEBs only** — Windows MSI / macOS PKG ship the core stub). Packages **do not vendor** `rcl`/RMW/DDS — install system ROS and `source /opt/ros/<distro>/setup.bash` |
| Broker | Running `robot_bus_broker` reachable over tcp/ipc (or `bus_discover`) |
| Topic types | **Registry** — configure any registered type by string (not a hardcoded enum). Built-in: `std_msgs/msg/String`, `sensor_msgs/msg/Imu`, `sensor_msgs/msg/Image`, `foxglove_msgs/msg/CompressedVideo`. Extend by implementing `TopicCodec` + registering it. |
| Service types | `std_srvs/srv/Trigger`, `std_srvs/srv/SetBool` (directions `ros_to_bus` / `bus_to_ros` only; default call timeout 5s) |
| Action types | `example_interfaces/action/Fibonacci` (directions `ros_to_bus` / `bus_to_ros` only; default goal timeout 30s) |

```rust
use robot_bus::ros2::{Direction, Ros2Bridge};

let mut bridge = Ros2Bridge::new("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
        .string()
        .direction(Direction::Both)
        .add()?
    .route("/camera/image_raw", "/camera/image_raw")
        .type_name("sensor_msgs/msg/Image")
        .direction(Direction::RosToBus)
        .add()?
    .service("/reset", "/reset")
        .trigger()
        .direction(Direction::RosToBus)
        .add()?
    .service("/enable", "/enable")
        .set_bool()
        .direction(Direction::BusToRos)
        .add()?
    .action("/fibonacci", "/fibonacci")
        .fibonacci()
        .direction(Direction::RosToBus)
        .add()?
    .build()?;
bridge.spin()?;
// or: Ros2Bridge::from_yaml("bridge.yaml")?.spin()?;
// Camera→H264 example YAML: src/ros2/example_camera_h264.yaml
```

`foxglove_msgs/msg/CompressedVideo` requires the `foxglove_msgs` ROS package on the system (DynamicMessage type support).

C++ (after installing the matching **Linux** `robot-bus-ros2-*` package and sourcing ROS):

```cpp
#include <robot_bus/Ros2Bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .string()
    .direction(robot_bus::Ros2Direction::Both)
    .add()
    .route("/camera/image_raw", "/camera/image_raw")
    .type_name("sensor_msgs/msg/Image")
    .direction(robot_bus::Ros2Direction::RosToBus)
    .add()
    .service("/reset", "/reset")
    .trigger()
    .direction(robot_bus::Ros2Direction::RosToBus)
    .add()
    .action("/fibonacci", "/fibonacci")
    .fibonacci()
    .direction(robot_bus::Ros2Direction::RosToBus)
    .add()
    .build();
bridge.spin();
// or: robot_bus::Ros2Bridge::from_yaml("bridge.yaml").spin();
// Camera→H264 example: src/ros2/example_camera_h264.yaml
```

See [`docs/cpp-api.md`](docs/cpp-api.md) for package selection and local `just cpp-dev-ros2`.

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
- Typed `create_publisher::<M>` also **best-effort registers** `topic → M::full_name()` (e.g. `sensor_msgs.msg.v1.Imu`) with the broker console HTTP API so `rbus topic list` / `topic info` can show types without putting type metadata on the wire
- **srv** is a pair of `*Request` / `*Response` messages, not gRPC
- **grpc** (`robot_bus`) is the gateway RPC contract, started with the broker (default feature `grpc`)
- Messages live under the `robot_bus` namespace and do **not** claim top-level ROS package names like `sensor_msgs`; encoding is protobuf and is not interoperable with ROS CDR
- One-shot: `just gen-all`

Covered packages: `builtin_interfaces`, `std_msgs`, `std_srvs`, `geometry_msgs`, `sensor_msgs`, `nav_msgs`, `tf2_msgs`, `trajectory_msgs`, `diagnostic_msgs`, `unique_identifier_msgs`, `shape_msgs`, `visualization_msgs`, `control_msgs`, `nav2_msgs`, `apriltag_msgs`, `foxglove_msgs` (ported from [Foxglove schemas](https://github.com/foxglove/foxglove-sdk), package `foxglove_msgs.msg.v1`).
