[English](README.md) | 中文

# *Robot Bus*

[![CI](https://github.com/indunet/robot-bus/actions/workflows/ci.yml/badge.svg)](https://github.com/indunet/robot-bus/actions/workflows/ci.yml)
[![Code Quality](https://img.shields.io/github/actions/workflow/status/indunet/robot-bus/dynamic%2Fgithub-code-scanning%2Fcodeql?label=Code%20Quality)](https://github.com/indunet/robot-bus/security/code-scanning)
[![crates.io](https://img.shields.io/crates/v/robot-bus.svg?color=f74c00)](https://crates.io/crates/robot-bus)
[![PyPI](https://img.shields.io/pypi/v/robot-bus.svg?color=3775a9)](https://pypi.org/project/robot-bus/)
[![npm](https://img.shields.io/npm/v/robot-bus.svg?color=cb3837)](https://www.npmjs.com/package/robot-bus)
[![Maven Central](https://img.shields.io/maven-central/v/org.indunet/robot-bus.svg?label=Maven%20Central&color=007396)](https://central.sonatype.com/artifact/org.indunet/robot-bus)
[![License](https://img.shields.io/badge/License-Apache_2.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

轻量级、免环境配置的 ROS 2 风格通信库：基于 ZeroMQ，提供 topic / service / action，以及 `Executor` + `Node` + `spin` 回调模型。多语言 SDK 覆盖 Rust / Python / TypeScript / C++ / Java / Android。

不依赖 ROS 发行版、不需要 `source setup.bash`、不搭 workspace。一个 broker 进程 + 任一语言的 SDK 即可。

**设计原则**：API 会尽量贴近 ROS 2 的用法与命名（如 `Node`、`SingleThreadedExecutor` / `MultiThreadedExecutor`、`add_node`、`create_publisher` / `create_subscription`、`spin`），降低从 ROS 2 迁过来的心智负担；底层用 ZeroMQ 实现，不绑定某一 ROS 发行版。

> **预发布说明**：当前仍处于预发布阶段。接下来 API 可能会有较多变更，运行稳定性也尚不完善，请谨慎用于生产环境。

更多 API 示例见 [`docs/`](docs/)。

### Crate API

| 模块 | 职责 |
|------|------|
| `broker::` | 路由进程（message / service / action） |
| 顶层 API | Publisher / Subscriber / Client / Worker |
| `runtime::Executor` | 底层 poll loop（一般用下面两个包装） |
| `runtime::SingleThreadedExecutor` / `MultiThreadedExecutor` | 显式执行器（多节点 / 并行）；单节点可直接 `Node::spin` |
| `runtime::Node` / `TopicPublisher` / `CallbackGroup` | 节点、publisher、callback group（互斥 / 可重入） |
| `grpc::`（默认 feature） | gRPC / gRPC-Web 网关（随 broker 一起启动） |
| `ros2::`（`ros2` feature） | 进程内 ROS 2 话题/服务桥（`Ros2Bridge`） |

### 仓库布局

Rust 核心留在仓库根目录（`Cargo.toml` + `src/`）。各语言 SDK 放在 `bindings/` 下，不要拆成与 Rust 同级的顶层目录。

| 路径 | 职责 |
|------|------|
| [`src/`](src/)、`Cargo.toml` | Rust 核心（crates.io / maturin 入口） |
| [`proto/`](proto/) | 契约源：ROS 风格 Protobuf → Rust / bindings 生成代码 |
| [`bindings/`](bindings/) | 语言 SDK（Python、TypeScript、C++、Java、Android） |
| [`console/`](console/) | Web 监控控制台（产品 UI；构建产物同步到本地/CI 的 `assets/console/`，不入库） |
| [`benches/`](benches/) | 性能压测：[`robot_bus_perf/`](benches/robot_bus_perf/)（`just perf`）、[`ros2_perf/`](benches/ros2_perf/)（`just perf-ros2`） |
| [`tests/`](tests/) | Rust 集成测试 + 跨语言互通（`just test-interop`） |
| [`docs/`](docs/) | API 文档与生成的性能报告 |
| [`scripts/`](scripts/)、[`tools/`](tools/)、`justfile` | 代码生成、打包与任务编排 |

## 架构

```
业务代码 (Rust / Python / TypeScript / C++ / Java / Android)
  └── robot-bus SDK
              │
              │ ZMQ (tcp / ipc / inproc) 或 gRPC / gRPC-Web
              ▼
robot_bus_broker 进程
```

### 可选 ROS 2 桥（Rust feature）

日常开发 **不必安装 ROS 2**。进程内与 ROS 2 图互通时，开启 Cargo feature **`ros2`**，使用 `robot_bus::ros2::Ros2Bridge`（链式 API 或 YAML）。官方支持：**Humble**、**Jazzy**（需 source 对应发行版并链接 `rclrs`）。C++ 另有 `robot-bus-ros2-humble` / `…-jazzy` 包，依赖系统 ROS，**不**把 `rcl` 打进安装包。见 [ROS 2 桥](#ros-2-桥-feature-ros2)。

## 快速开始

### 1. 启动 broker

Rust：

```bash
cargo run --bin robot_bus_broker
# 发现 / domain: robot_bus_broker --domain-id 0 --advertise-host 10.0.0.5
# 关闭广播:        robot_bus_broker --no-discovery
```

### 自省 CLI（`rbus`）

查询 broker console 的 HTTP API（默认 `http://127.0.0.1:15771`；可用 `--url` 或环境变量 `ROBOT_BUS_BROKER_URL` 覆盖）：

```bash
cargo run --bin rbus -- topic list
cargo run --bin rbus -- service list
cargo run --bin rbus -- action list
cargo run --bin rbus -- status
```

topic list 只显示近期有转发流量的名字（metrics 需要真实订阅者）。service / action 在 worker READY 后出现。

### Broker 发现（UDP 组播）

Broker 周期在 `239.255.76.67:15550` 上广播（刻意避开 ROS 2 / DDS 的 `7400` 与 `239.255.0.1`）。UDP 载荷为纯 protobuf [`BrokerAnnounce`](proto/robot_bus_interface/msg/v1/announce.proto)（`magic` 必须为 `RBUS`）；解不出来的包直接丢弃。

Client **仍手动选择传输**（`tcp` / `ipc` / `inproc` / `grpc`）；发现只填充 host / 路径 / gRPC URL：

```rust
use robot_bus::{DiscoverOpts, Node, NodeOptions};

let opts = NodeOptions::tcp().discover(DiscoverOpts {
    domain_id: 0,
    ..Default::default()
})?;
let mut node = Node::with_options("talker", opts);
```

各语言绑定同款：`Node.discover(...)`（Python / C++ / Java / Android / TypeScript Node.js）。浏览器 gRPC-Web 无 UDP 发现。

同 domain 多 broker 时请指定 `broker_id`，否则发现会报错并列出候选 id。

Python（`pip install robot-bus` 后自带可执行入口）：

```bash
robot-bus-broker
```

或在代码里进程内启动：

```python
import robot_bus

with robot_bus.RobotBusBroker.start() as broker:
    # ... 业务代码 ...
    pass
# 离开 with 自动 stop

# 或阻塞运行（等同于命令行，Ctrl+C 退出）
# robot_bus.run_broker()
```

### Python

```bash
pip install robot-bus
```

本地开发（需 [maturin](https://www.maturin.rs/)，可选 [just](https://github.com/casey/just)）：

```bash
just python-dev
# 等价：cd bindings/python && maturin develop --features extension-module,grpc
```

（`grpc` 为默认 feature；显式写出可避免 `default-features = false` 的构建漏掉网关。）

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
# node.spin()  # 阻塞；另线程调用 node.shutdown() / shutdown_handle().shutdown()
```

（不传消息类型时仍为 raw bytes。多节点共享或需多线程 handler 时再用 `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node`。）

仅走 gRPC 网关时：`Node.grpc("name")` / `Node.grpc_at("name", "http://…")`（客户端：订阅 / publish / 调 service / action）。详见 [`docs/python-api.md`](docs/python-api.md)。

### TypeScript

```bash
npm install robot-bus
```

本地开发：

```bash
just ts-dev
# 等价：cd bindings/typescript && npm install && npm run build:native && npm run build:ts
```

单一 npm 包：Node.js 走 napi-rs（完整 ZMQ API）；浏览器走 gRPC-Web（订阅 / publish / service / action）。bundler 通过 `exports` 自动选入口。详见 [`docs/typescript-api.md`](docs/typescript-api.md)。

```ts
import { Node } from "robot-bus";
import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";

const node = new Node("pilot");
const pub = node.createPublisher("/robot1/imu", Imu);
node.createSubscription("/robot1/imu", (_t, imu) => console.log(imu), Imu);
```

浏览器 / 纯 gRPC：`Node.grpc("client")`（browser 入口的 `Node` 即为 gRPC-Web facade）。

### Java / Android（Maven Central）

| 产物 | 目录 | 坐标 |
|------|------|------|
| JVM JAR（Java 11+，Maven） | [`bindings/java/`](bindings/java/) | `org.indunet:robot-bus` |
| Android AAR（minSdk 24，Kotlin SDK） | [`bindings/android/`](bindings/android/) | `org.indunet:robot-bus-android` |

包名均为 `org.indunet.robot.bus`。Android 是**独立** Kotlin SDK（不依赖 Java JAR）。在 GitHub 上写 Release 说明并 Publish 后，CI 会发到 Maven Central（也可手动跑 Actions）。

```bash
just java-dev       # JVM
just android-dev    # AAR（需 Android SDK + NDK 26 + cargo-ndk）
```

```kotlin
// Android（Kotlin）
RobotBusAndroid.init(this)
val pub = node.createPublisher("/imu", Imu::class.java)
```

详见 [`docs/java-api.md`](docs/java-api.md) / [`docs/android-api.md`](docs/android-api.md)、[`bindings/java/README.md`](bindings/java/README.md) / [`bindings/android/README.md`](bindings/android/README.md)。

### C++（DEB / MSI）

C++ 无中央库：从 [GitHub Releases](https://github.com/indunet/robot-bus/releases) 下载（Publish 后 CI 挂附件）：

| 包 | 内容 |
|----|------|
| `robot-bus_*_linux_*.deb`（另有 MSI / PKG） | 核心 SDK + broker，**无** ROS 2 桥 |
| `robot-bus-ros2-humble_*_linux_*.deb` | 同上 + **Humble** 桥（**仅 Linux**；需系统 Humble；不 vendor `rcl`） |
| `robot-bus-ros2-jazzy_*_linux_*.deb` | 同上 + **Jazzy** 桥（**仅 Linux**） |

三选一安装（互斥）。详见 [`docs/cpp-api.md`](docs/cpp-api.md)。

```cpp
#include <robot_bus/Node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

robot_bus::Broker broker;
robot_bus::Node node("pilot");
auto pub = node.create_publisher("/imu");
```

### Rust（Node + spin）

在 `Cargo.toml` 中添加依赖：

```toml
robot-bus = { path = "../robot-bus" }
# 或 crates.io：robot-bus = "0.1.2"
```

语义接近 ROS 2：`Node::new` → typed `create_publisher` / `create_subscription` → `node.spin()`（自动挂 `SingleThreadedExecutor`）。

仅走 gRPC 网关（不启 ZMQ）时用 `Node::grpc` / `Node::grpc_at`：可订阅、publish、调 service / action，不能当 server；详见 [`docs/rust-api.md`](docs/rust-api.md#grpc-模式-node客户端)。

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
        // 控制周期 / 心跳
    }),
    None,
)?;

let handle = node.shutdown_handle()?;
std::thread::spawn(move || { /* ... */ handle.shutdown(); });
node.spin()?;
```

- 单节点默认：直接 `node.spin()`（内部 `SingleThreadedExecutor`）
- `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node`：多节点共享或并行 handler
- Callback group：`MutuallyExclusive` / `Reentrant`（`create_callback_group`；默认互斥组）
- Service / action：typed `create_service` / `create_client`、`create_action_server` / `create_action_client`（与 topic 一样挂在 Node；另有 `*_raw`）
- Timer：`create_timer`（同样挂在 Node，由 `spin` 驱动）
- Raw bytes：`create_publisher_raw` / `create_subscription_raw`
- 底层 escape hatch：`Executor`（高级用法）

发送 / 接收水位（ZMQ HWM，不是完整 QoS）可在创建时或运行中设置：

```rust
use robot_bus::{Publisher, HighWaterMark};

let pub_ = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
pub_.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

默认：message `STREAM(2/2)`、service `RPC(4/4)`、action `ACTION(8/8)`。Broker 侧用 `--snd-hwm` / `--rcv-hwm`。

## 二进制

| 二进制 | 说明 |
|------|------|
| `robot_bus_broker` | 一次启动三条总线 + gRPC / gRPC-Web 网关 |

## Web 控制台（`console/`）

可选监控前端：查看 broker 状态、topic 流量与事件日志。开启 `console` feature（默认）时，先构建一次静态资源后，broker 在 `0.0.0.0:15771` 提供**嵌入式** UI。

**开发（热更新，推荐）：**

```bash
# 终端 1
cargo run --bin robot_bus_broker
# 终端 2
cd console && pnpm install && pnpm dev
# 打开 http://localhost:3000  （/api 会代理到 broker；可用 ROBOT_BUS_BROKER_URL 覆盖）
```

**嵌入 broker 二进制：**

```bash
just console          # pnpm build + sync → assets/console/（已 gitignore）
cargo run --bin robot_bus_broker
# 打开 http://localhost:15771
# 关闭：cargo run --bin robot_bus_broker -- --no-console
```

`assets/console/` 是**构建产物**（不提交）。CI / 发布在带 `console` feature 编译前会先生成该目录。

已对接 broker 同端口监控 API：`GET /api/v1/status`、`GET /api/v1/topics`、`GET /api/v1/services`、`GET /api/v1/actions`、`SSE /api/v1/events`。前端源码在 `console/`；只有生成的静态文件会编进带 `console` feature 的二进制。

## gRPC / gRPC-Web 网关

随 `robot_bus_broker` / `RobotBusBroker::start` 一起启动；标准 gRPC 与 gRPC-Web **同端口**（默认 `0.0.0.0:15770`）。

也可用 `Node::grpc` / `Node::grpc_at` 以 Node API 接入网关（客户端：订阅 / publish / 调 service / 调 action，见 [`docs/rust-api.md`](docs/rust-api.md#grpc-模式-node客户端)）。

| RPC | 语义 |
|-----|------|
| `MessageGateway.Subscribe` | 按 topic 前缀订阅，服务端流式返回二进制 payload |
| `MessageGateway.Publish` | 一元发布：topic + 二进制 payload 写入 message bus |
| `ServiceGateway.Call` | 一元：`service_name` + request bytes → response bytes |
| `ActionGateway.Run` | 双向流：客户端发 GOAL / CANCEL，服务端推 `ActionEvent`（`kind` 区分 FEEDBACK / RESULT） |

```bash
cargo run --bin robot_bus_broker
# 配置：cargo run --bin robot_bus_broker -- --help
# gRPC: http://0.0.0.0:15770
```

进程内：

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

Proto（包名 `robot_bus_interface.grpc.v1`，与 ROS `*.msg.v1` / `*.srv.v1` 区分）：

- [`message_gateway.proto`](proto/robot_bus_interface/grpc/v1/message_gateway.proto)
- [`service_gateway.proto`](proto/robot_bus_interface/grpc/v1/service_gateway.proto)
- [`action_gateway.proto`](proto/robot_bus_interface/grpc/v1/action_gateway.proto)

UDP 发现（包名 `robot_bus_interface.msg.v1`）：

- [`announce.proto`](proto/robot_bus_interface/msg/v1/announce.proto)

## 工具节点（Cargo features）

工具二进制作为主 crate `robot-bus` 的 **默认 feature** 提供。先装系统依赖（FFmpeg / ALSA 头文件），再：

```bash
cargo install robot-bus --bin robot_bus_image_encoder
cargo install robot-bus --bin robot_bus_audio_capture
cargo install robot-bus --bin robot_bus_audio_play
cargo install robot-bus --bin robot_bus_camera_capture
```

只要库时用 `--no-default-features --features grpc,console`。
### 图像编码节点（`robot_bus_image_encoder`）

订阅 `sensor_msgs/Image`（`rgb8` / `bgr8` / `mono8`），经 **系统 FFmpeg** 发布为 `foxglove_msgs/CompressedVideo`（`h264` 或 `h265`，Annex-B）。编码器优先级：NVENC → VideoToolbox → `libopenh264` / 软编。

```bash
# macOS
brew install ffmpeg
# Debian/Ubuntu
sudo apt install ffmpeg libavcodec-dev libavutil-dev libswscale-dev

cargo install robot-bus --bin robot_bus_image_encoder
robot_bus_image_encoder --print-example-config > encoder.yaml
robot_bus_image_encoder --params encoder.yaml
```

若链接 GPL 软编（`libx264` / `libx265`），由部署方自行合规；有硬件编码时优先硬编。

### 音频采集（`robot_bus_audio_capture`）

经 [cpal](https://github.com/RustAudio/cpal) 以**共享**（非独占）模式采集麦克风 PCM，发布 `foxglove_msgs/RawAudio`（`pcm-s16`）。默认：16 kHz 单声道、20 ms 分块。feature `audio-capture`（默认开）。

```bash
# Debian/Ubuntu
sudo apt install libasound2-dev

cargo install robot-bus --bin robot_bus_audio_capture
robot_bus_audio_capture --list-devices
robot_bus_audio_capture --print-example-config > capture.yaml
robot_bus_audio_capture --params capture.yaml
```

### 音频播放（`robot_bus_audio_play`）

订阅 `foxglove_msgs/RawAudio`（`pcm-s16`），经 cpal 共享模式输出到扬声器。feature `audio-play`（默认开）。消息中的采样率 / 声道须与节点参数一致。

```bash
cargo install robot-bus --bin robot_bus_audio_play
robot_bus_audio_play --list-devices
robot_bus_audio_play --print-example-config > play.yaml
robot_bus_audio_play --params play.yaml
```

### 相机采集（`robot_bus_camera_capture`）

经 [nokhwa](https://github.com/l1npengtul/nokhwa)（V4L2 / AVFoundation / Media Foundation）采集 USB / 摄像头画面，发布 `sensor_msgs/Image`（`rgb8`）。默认：640×480 @ 30 fps，话题 `/camera/image_raw`，可直接接 `robot_bus_image_encoder`。feature `camera-capture`（默认开）。macOS 需在弹窗中授权摄像头。

```bash
cargo install robot-bus --bin robot_bus_camera_capture
robot_bus_camera_capture --list-devices
robot_bus_camera_capture --print-example-config > camera.yaml
robot_bus_camera_capture --params camera.yaml
```

## ROS 2 桥（`feature = "ros2"`）

进程内话题**与**服务桥：`robot_bus::ros2::Ros2Bridge`（链式 API 或 YAML）。**默认不启用** — 核心 SDK / crates.io / maturin 仍免 ROS。

**官方支持的 ROS 2 发行版：** **Humble**、**Jazzy**。其它发行版：source 后本机自建（best-effort）。

| 需要 | 说明 |
|------|------|
| Cargo（Rust） | `--features ros2`（可选依赖 `rclrs`） |
| 环境 | Source **Humble** 或 **Jazzy** 以便链接 `rcl`；主 CI **不**开此 feature |
| C++ 包 | `robot-bus`（无桥）与 `robot-bus-ros2-humble` / `robot-bus-ros2-jazzy`（互斥，**仅 Linux DEB** — Windows MSI / macOS PKG 只有核心 stub）。包内 **不 vendor** `rcl`/RMW/DDS — 需安装系统 ROS 并 `source /opt/ros/<distro>/setup.bash` |
| Broker | 可达的 `robot_bus_broker`（tcp/ipc 或 `bus_discover`） |
| MVP 话题类型 | `std_msgs/msg/String`、`sensor_msgs/msg/Imu` |
| MVP 服务类型 | `std_srvs/srv/Trigger`、`std_srvs/srv/SetBool`（仅 `ros_to_bus` / `bus_to_ros`；默认调用超时 5s） |

```rust
use robot_bus::ros2::{Direction, Ros2Bridge};

let mut bridge = Ros2Bridge::new("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
        .string()
        .direction(Direction::Both)
        .add()
    .service("/reset", "/reset")
        .trigger()
        .direction(Direction::RosToBus)
        .add()?
    .service("/enable", "/enable")
        .set_bool()
        .direction(Direction::BusToRos)
        .add()?
    .build()?;
bridge.spin()?;
// 或: Ros2Bridge::from_yaml("bridge.yaml")?.spin()?;
```

C++（安装对应的 **Linux** `robot-bus-ros2-*` 并 source ROS 后）：

```cpp
#include <robot_bus/Ros2Bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .string()
    .direction(robot_bus::Ros2Direction::Both)
    .add()
    .service("/reset", "/reset")
    .trigger()
    .direction(robot_bus::Ros2Direction::RosToBus)
    .add()
    .build();
bridge.spin();
// 或: robot_bus::Ros2Bridge::from_yaml("bridge.yaml").spin();
```

详见 [`docs/cpp-api.md`](docs/cpp-api.md)；本机构建可用 `just cpp-dev-ros2`。

## 测试

```bash
just test-rust
just test-python
just test-typescript
just test-interop   # 跨语言矩阵，见 tests/interop/
just perf           # robot-bus → docs/perf-report.md（benches/robot_bus_perf/）
just perf-ros2      # ROS 2 对标，见 benches/ros2_perf/
# 等价：
# cargo test
# PYTHONPATH=bindings/python python3 bindings/python/tests/test_msgs_roundtrip.py
# PYTHONPATH=bindings/python python3 bindings/python/tests/test_typed_api.py
# cd bindings/typescript && npm test
```

## Protobuf 消息

[`proto/`](proto/) 按 ROS 包布局：`proto/<pkg>/{msg|srv|grpc}/v1/*.proto`。

各语言 stub **不进 git**；本地改 proto 或跑测试前执行 `just gen-*`（需 protoc **35.1**）。CI / 发版流水线会生成并打进 wheel、crates.io crate、npm 包、DEB/MSI、Maven JAR/AAR——**消费已发布包的用户不需要 protoc**。

| 语言 | 路径 | 说明 |
|------|------|------|
| Rust | `robot_bus::<pkg>::{msg\|srv}::v1` | `just gen-rust` → `src/generated/<pkg>/{msg\|srv}/v1/<stem>.rs` |
| Python | `robot_bus.<pkg>.{msg\|srv}.v1` | `just gen-python`；随 wheel 打包 |
| TypeScript | `robot-bus/<pkg>/{msg\|srv}/v1/…` | `just gen-typescript`；随 npm 包打包 |
| Java / Android | `org.indunet.robot.bus.<pkg>.{msg\|srv\|action}.v1` | `just gen-java`；随 JAR / AAR 打包 |
| C++ | `#include <robot_bus/…>` | `just gen-cpp`；随 DEB/MSI 打包 |

- 传输层 body 仍是 opaque bytes（含 gRPC 网关）；Rust Node SDK 在 create 时绑定类型并自动 encode/decode（`create_publisher::<M>` 等），也可用 `*_raw`；Python / TypeScript / **Java** 传入 protobuf 类型即可 typed（薄封装），省略类型则为 raw bytes
- **srv** 是一对 `*Request` / `*Response` message，不是 gRPC
- **grpc**（`robot_bus`）是网关 RPC 契约，随 broker 启动（默认 feature `grpc`）
- 消息在 `robot_bus` 命名空间下，**不占用** ROS 顶层 `sensor_msgs` 包名；编码是 protobuf，与 ROS CDR 不互通
- 一键：`just gen-all`

已覆盖：`builtin_interfaces`、`std_msgs`、`std_srvs`、`geometry_msgs`、`sensor_msgs`、`nav_msgs`、`tf2_msgs`、`trajectory_msgs`、`diagnostic_msgs`、`unique_identifier_msgs`、`shape_msgs`、`visualization_msgs`、`control_msgs`、`nav2_msgs`、`foxglove_msgs`（自 [Foxglove schemas](https://github.com/foxglove/foxglove-sdk) 迁入，包名为 `foxglove_msgs.msg.v1`）。
