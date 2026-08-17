English | [中文](README-zh.md)

# *Robot Bus*

[![CI](https://github.com/indunet/robot-bus/actions/workflows/ci.yml/badge.svg)](https://github.com/indunet/robot-bus/actions/workflows/ci.yml)
[![Code Quality](https://img.shields.io/github/actions/workflow/status/indunet/robot-bus/dynamic%2Fgithub-code-scanning%2Fcodeql?label=Code%20Quality)](https://github.com/indunet/robot-bus/security/code-scanning)
[![crates.io](https://img.shields.io/crates/v/robot-bus.svg?color=f74c00)](https://crates.io/crates/robot-bus)
[![PyPI](https://img.shields.io/pypi/v/robot-bus.svg?color=3775a9)](https://pypi.org/project/robot-bus/)
[![npm](https://img.shields.io/npm/v/robot-bus.svg?color=cb3837)](https://www.npmjs.com/package/robot-bus)
[![Maven Central](https://img.shields.io/maven-central/v/org.indunet/robot-bus.svg?label=Maven%20Central)](https://central.sonatype.com/artifact/org.indunet/robot-bus)
[![License](https://img.shields.io/badge/license-Apache%202.0-4EB1BA.svg)](https://www.apache.org/licenses/LICENSE-2.0.html)

Robot Bus is a lightweight, multi-language messaging **framework** with a ROS 2–style programming model — topics, services, actions, and `Node` + `spin` — built on ZeroMQ. It does not replace ROS 2; it extends the ROS ecosystem to platforms and languages where a full ROS 2 stack is difficult to deploy or heavier than needed (for example Android, Windows, and browser clients).

SDKs: **Rust**, **Python**, **TypeScript**, **C++**, **Java**, **Android**.

![Robot Bus Web console](docs/images/console-overview.png)

*Web console* — Overview / Topics / Services / Actions / Topology. Start `robot-bus-broker`, then open [http://127.0.0.1:15570](http://127.0.0.1:15570). See [§4 Web console](#4-web-console) and the [Tank demo](#32-tank-demo).

## *Design Philosophy*

- **ROS 2 model, lighter runtime:** Topics, services, actions, and `Node` + `spin` without a ROS distro, `source setup.bash`, or a workspace — one broker process plus an SDK is enough.
- **APIs that migrate:** Naming and usage stay close to ROS 2 so working code can later become a ROS 2 node, or stay on robot-bus and join an existing graph through the [ROS 2 bridge](#5-ros-2-bridge).
- **Grounded in deployment reality:** The same programming model on Android, Windows, browsers, and other hosts where a full ROS 2 stack is impractical.

## *Key Features*

- **ROS 2–style primitives:** Topic pub/sub, service, action (`send_goal` → GoalHandle → `result` / `cancel`), timers, and parameters.
- **One broker, many languages:** Rust, Python, TypeScript, C++, Java, and Android SDKs against the same bus.
- **Embedded Web console:** Overview, Topics, Services, Actions, Topology, plus a built-in tank demo — no extra frontend process.
- **Optional ROS 2 bridge:** In-process topic / service / action bridging with `rclrs` / `rclpy` / `rclcpp` (Humble / Jazzy). The core SDK stays ROS-free unless the bridge is enabled.
- **Protobuf contracts:** All payloads are Protocol Buffers (not ROS CDR), aligned with common ROS 2 package names under [`proto/`](proto/).
- **Browser and remote clients:** WebSocket RPC (`Node::ws` / `Node.ws`; transport `"ws"`). `"grpc"` and `--grpc-listen` remain aliases.

The Node programming model — `Context` / `Node`, topic pub-sub, service, action, and `spin` — is the stable public API.

### *Documentation*

- Language guides: [`docs/en/`](docs/en/) (Python / Rust / TypeScript / C++ / Java / Android)
- ROS 2 bridge: [`docs/en/ros2-bridge.md`](docs/en/ros2-bridge.md)
- Migration playbooks: [`docs/skills/`](docs/skills/)
- Performance: [`docs/en/perf-report.md`](docs/en/perf-report.md)

### *Install*

* Python (includes the `robot-bus-broker` CLI)

```bash
pip install robot-bus
```

* Rust

```toml
robot-bus = "1.2.1"
```

* npm

```bash
npm install robot-bus
```

* Maven

```xml
<dependency>
    <groupId>org.indunet</groupId>
    <artifactId>robot-bus</artifactId>
    <version>1.2.1</version>
</dependency>
```

C++ packages (DEB / MSI) and Android (`org.indunet:robot-bus-android`) are listed in [§3.6](#36-other-languages).


## *1. Application scenarios*

### *1.1 Lightweight ROS 2–style messaging*

When a full ROS 2 installation is unnecessary, robot-bus provides the same programming model with a smaller footprint — suitable for prototypes, tooling, Windows hosts, and constrained deployments.

### *1.2 Heterogeneous systems with ROS 2*

Run ROS 2 on Ubuntu (or other Linux hosts) as usual, and place part of the compute on Android devices or other hosts where ROS 2 is impractical. Use robot-bus on those hosts with the same topic / service / action model, then interconnect via the ROS 2 bridge.

### *1.3 Prototype on bus, migrate to ROS 2*

Because robot-bus is lightweight and quick to bring up, teams can prototype and validate nodes on bus first, then migrate the validated design to native ROS 2 (or keep the process on bus and bridge only the interfaces that must join the ROS 2 graph).

Migration playbooks for Agent / developers: [`docs/skills/ros2-to-robot-bus`](docs/skills/ros2-to-robot-bus/SKILL.md) and [`docs/skills/robot-bus-to-ros2`](docs/skills/robot-bus-to-ros2/SKILL.md). In Cursor, `@` those files or ask to migrate a package either way.

## *2. Architecture*

```
  Application (Python / Rust / C++ / Java / Android / …)
                    │
                    │  ZMQ (tcp / ipc / inproc) or WebSocket RPC
                    ▼
             robot_bus_broker
                    │
                    │  optional ros2_bridge (rclrs / rclpy / rclcpp)
                    ▼
               ROS 2 graph
```

## *3. Quick start*

### *3.1 Install and start the broker*

```bash
pip install robot-bus
robot-bus-broker
```

Default API / Web console / WebSocket listen: `http://0.0.0.0:15570`. After the broker is up, open the [Web console](#4-web-console) in a browser.

Runnable demos (topic, service, action) for Rust / Python / C++: [`examples/`](examples/).

Or start the broker in-process:

```python
import robot_bus

with robot_bus.RobotBusBroker.start() as broker:
    # application code …
    pass
```

### *3.2 Tank demo*

![Tank sim in the Web console](docs/images/tank-sim.png)

A built-in mini tank sim helps you see topics moving end-to-end without writing code first:

1. Start the broker (`robot-bus-broker`).
2. Open **http://127.0.0.1:15570** and click **TANK** in the sidebar (or go to `/tank/`).
3. Click the panel, then drive with **arrow keys**; or switch to point navigation and **click on the map** to send a goal.

Opening the panel starts the in-process tank node. It subscribes to `/robot_bus/tank/cmd_vel` and publishes `/robot_bus/tank/pose`. Multiple browsers share one world (teleop is last-writer-wins). Disable with `--no-tank` if needed.

### *3.3 Topic (publish / subscribe)*

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
# node.spin()
```

### *3.4 Service*

```python
import robot_bus
from robot_bus.std_srvs.srv.v1 import SetBoolRequest, SetBoolResponse

def on_set_bool(req: SetBoolRequest) -> SetBoolResponse:
    return SetBoolResponse(success=True, message=f"set:{req.data}")

server = robot_bus.Node("worker")
client = robot_bus.Node("caller")

server.create_service(
    "/set_bool", on_set_bool,
    request_type=SetBoolRequest, response_type=SetBoolResponse,
)
svc = client.create_client(
    "/set_bool",
    request_type=SetBoolRequest, response_type=SetBoolResponse,
)
# reply = svc.call(SetBoolRequest(data=True), timeout=5.0)
# server.spin()
```

### *3.5 Action*

```python
import robot_bus
from robot_bus.example_interfaces.action.v1 import (
    FibonacciGoal, FibonacciFeedback, FibonacciResult,
)

def on_fibonacci(goal: FibonacciGoal, context):
    seq = list(range(goal.order))
    context.publish_feedback(FibonacciFeedback(sequence=seq[:1]))
    return FibonacciResult(sequence=seq)

server = robot_bus.Node("worker")
client = robot_bus.Node("caller")

server.create_action_server(
    "/fibonacci", on_fibonacci,
    goal_type=FibonacciGoal,
    feedback_type=FibonacciFeedback,
    result_type=FibonacciResult,
)
act = client.create_action_client(
    "/fibonacci",
    goal_type=FibonacciGoal,
    feedback_type=FibonacciFeedback,
    result_type=FibonacciResult,
)
goal = act.send_goal(
    FibonacciGoal(order=5),
    feedback_callback=lambda fb: print(fb.sequence),
)
# result = goal.result(timeout=10.0)
# server.spin()
```

More detail: [`docs/en/python-api.md`](docs/en/python-api.md).

### *3.6 Other languages*

| Language | Package / artifact | Guide |
|----------|--------------------|-------|
| Python | [PyPI `robot-bus`](https://pypi.org/project/robot-bus/) | [`docs/en/python-api.md`](docs/en/python-api.md) |
| Rust | [crates.io `robot-bus`](https://crates.io/crates/robot-bus) | [`docs/en/rust-api.md`](docs/en/rust-api.md) |
| TypeScript | [npm `robot-bus`](https://www.npmjs.com/package/robot-bus) | [`docs/en/typescript-api.md`](docs/en/typescript-api.md) |
| C++ | [GitHub Releases](https://github.com/indunet/robot-bus/releases) (DEB / MSI) | [`docs/en/cpp-api.md`](docs/en/cpp-api.md) |
| Java | Maven Central `org.indunet:robot-bus` | [`docs/en/java-api.md`](docs/en/java-api.md) |
| Android | Maven Central `org.indunet:robot-bus-android` | [`docs/en/android-api.md`](docs/en/android-api.md) |
| ROS 2 bridge | per-language (`rclrs` / `rclpy` / `rclcpp`) | [`docs/en/ros2-bridge.md`](docs/en/ros2-bridge.md) |

## *4. Web console*

The broker ships with an embedded monitoring UI (Overview, Topics, Services, Actions, Topology, logs) — see the screenshot at the top of this README. After `robot-bus-broker` (or `RobotBusBroker.start()`), open:

**http://127.0.0.1:15570**

For a hands-on walkthrough, try the [Tank demo](#32-tank-demo) from the sidebar **TANK** entry. Same port as the API / WebSocket gateway. Disable the UI with `--no-console` if needed. Frontend source: [`console/`](console/); local UI development: [`console/README.md`](console/README.md).

## *5. ROS 2 bridge*

In-process topic / service / action bridging between robot-bus and ROS 2. Each language uses its native client (`rclrs` / `rclpy` / `rclcpp`). Official support: **Humble** and **Jazzy**. The core SDK stays ROS-free unless the bridge is enabled.

Requires a sourced ROS 2 distro and `rclpy`, plus a running broker:

```bash
source /opt/ros/humble/setup.bash   # or jazzy
robot-bus-broker                    # another terminal
```

```python
import robot_bus
from robot_bus.ros2_bridge import (
    Direction,
    Ros2Bridge,
    StdMsgsStringMapper,
    TriggerServiceMapper,
)

assert robot_bus.ros2_available()

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .mapper(StdMsgsStringMapper())
    .direction(Direction.Ros2ToBus)
    .add()
    .service("/reset", "/reset")
    .mapper(TriggerServiceMapper())
    .add()
    .build()
)
bridge.spin()
```

Full guide and examples (Rust / Python / C++): [`docs/en/ros2-bridge.md`](docs/en/ros2-bridge.md). For full package migration (not just bridging), see [`docs/skills/`](docs/skills/).

## *6. Protobuf messages*

All robot-bus payloads — **topics**, **services**, and **actions** — are defined and serialized with **Protocol Buffers**. The wire format is protobuf bytes (not ROS CDR). Typed APIs bind a protobuf message class at create time and encode/decode automatically; omit the type to work with raw bytes.

Contracts live under [`proto/`](proto/) in a ROS-style layout, aligned with common ROS 2 package names:

```text
proto/<package>/{msg|srv|action|grpc}/v1/*.proto
```

| Kind | How it is modeled |
|------|-------------------|
| Topic | A single `*.msg` protobuf message |
| Service | A pair of `*Request` / `*Response` messages under `*.srv` |
| Action | Goal / Feedback / Result messages under `*.action` |

Many built-in types are already provided, aligned with common ROS 2 packages. A few examples:

| Kind | ROS 2 | robot-bus |
|------|-------|-----------|
| Topic | `sensor_msgs/msg/Imu` | `robot_bus.sensor_msgs.msg.v1.Imu` |
| Topic | `geometry_msgs/msg/Twist` | `robot_bus.geometry_msgs.msg.v1.Twist` |
| Topic | `nav_msgs/msg/Odometry` | `robot_bus.nav_msgs.msg.v1.Odometry` |
| Service | `std_srvs/srv/SetBool` | `robot_bus.std_srvs.srv.v1.SetBoolRequest` / `SetBoolResponse` |
| Action | `example_interfaces/action/Fibonacci` | `robot_bus.example_interfaces.action.v1.FibonacciGoal` / … |
| Topic | `tf2_msgs/msg/TFMessage` | `robot_bus.tf2_msgs.msg.v1.TFMessage` |

Generated stubs ship inside published packages (PyPI, crates.io, npm, DEB/MSI, Maven) — consumers do not need `protoc`. Message modules live under the `robot_bus` namespace and do not claim top-level ROS package names on the wire. Full list: [`proto/`](proto/).

### *6.1 Custom messages*

When the builtins are not enough, define your own protobuf types the same way. Typed APIs accept any protobuf message class (they do not have to live in this repository).

1. Write a `.proto` (ROS-style package path recommended):

```protobuf
syntax = "proto3";
package my_robot.msg.v1;

message BatteryStatus {
  double voltage = 1;
  double percentage = 2;
}
```

2. Generate code into your own project, for example with Python:

```bash
protoc --python_out=. --pyi_out=. my_robot/msg/v1/battery_status.proto
```

3. Use it on a Node like a built-in type:

```python
from my_robot.msg.v1 import battery_status_pb2 as pb

node = robot_bus.Node("bms")
pub = node.create_publisher("/battery", pb.BatteryStatus)
node.create_subscription("/battery", lambda t, msg: print(msg.voltage), msg_type=pb.BatteryStatus)
pub.publish(pb.BatteryStatus(voltage=48.0, percentage=0.85))
```

To contribute a type into this repo’s built-in set: add the file under [`proto/`](proto/) and regenerate with `just gen-python` (or the matching `just gen-*` for other languages).

## *7. Contribution*

If you are interested in this project and want to join and undertake part of the work (development/testing/documentation),
please feel free to contact me via email <deng_ran@aliyun.com>

 *Robot Bus* is not built for profit. In restless moments, writing code brings me calm; if this library helps you, that is the motivation for me to keep refining it.


## *8. License*

Robot Bus is released under the [Apache 2.0 license](LICENSE).

```
Copyright 2026 indunet.org

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at the following link.

     http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
