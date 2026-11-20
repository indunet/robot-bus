English | [中文](../zh/ros2-bridge.md)

# ROS2 Bridge (`ros2_bridge`)

In-process bridge between **ROS 2** and **robot-bus**: Topic / Service / Action.

## Architecture: native per language

| Language | ROS client | Entry | Notes |
|----------|------------|-------|-------|
| **Rust** | `rclrs` | `robot_bus::ros2_bridge` (Cargo feature **`ros2`**) | Topics / services / actions all use typed `attach` (`TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper`) |
| **Python** | **`rclpy`** | `robot_bus.ros2_bridge` | Pure Python, **not** via Rust FFI / `rclrs` |
| **C++** | **`rclcpp`** | `<robot_bus/ros2_bridge.hpp>` + `robot_bus_ros2_bridge` | Native C++, **not** via Rust FFI / `rclrs` |

```text
Python:  rclpy  ──mapper──► robot_bus.Node
C++:     rclcpp ──mapper──► robot_bus::Node
Rust:    rclrs  ──mapper──► robot_bus::Node
```

**Why per language:** Topic / service / action all need compile-time concrete types (`create_subscription<T>`, `create_service<T>`, etc.). Each language creates ROS entities with concrete types on its side, then forwards via its own bus `Node`.

Target distributions: **Humble**, **Jazzy**. See [support and verification status](support.md) for build and runtime coverage. Mount routes with `.mapper(concrete object)`; custom mappers are field converters in that language.

---

## Prerequisites

```bash
source /opt/ros/humble/setup.bash   # or jazzy
# Prefer RobotBusBroker.start() in application code; CLI is for a standalone broker beside the bridge
cargo run --bin robot_bus_broker    # or python -m robot_bus.broker
```

**Rust `feature = "ros2"`** uses crates.io **`rclrs`**. Typed messages come from **`ros-env`**, which re-exports `share/<pkg>/rust/` on `AMENT_PREFIX_PATH`. Builtin topic mappers cover **Humble/Jazzy distro-common** interface packages only (`CORE_BRIDGE_PACKAGES` in `scripts/generate_topic_mappers.py`). Sourcing ROS is necessary but does not guarantee compatible rust IDL; verify generated field types and build in the target environment. Extension stacks (`nav2_msgs` / `control_msgs` / `foxglove_msgs` / `apriltag_msgs`) are **not** bridge builtins — write a Typed*Mapper (protos may still exist for bus-native use). See **Rust messages** below.

| Language | Dependencies |
|----------|--------------|
| Common | Reachable broker (tcp / ipc / discover) |
| Rust | `robot-bus = { features = ["ros2"] }` plus a sourced ROS env (typed messages below) |
| Python | `robot_bus` + system **`rclpy`** (`just python-dev` / `python-dev-ros2`) |
| C++ | `robot-bus-ros2-humble` or `…-jazzy`, or `just cpp-dev-ros2` (`-DROBOT_BUS_ROS2=ON`, links **rclcpp**) |

`ros2_available()`:

- **Python**: whether `import rclpy` succeeds
- **C++**: whether `robot_bus_ros2_bridge` was linked with `ROBOT_BUS_HAS_ROS2`
- **Rust FFI / default C ABI**: always false (bridge is not in FFI)

---

## Unified contract

Each topic, service, and action endpoint is a **name + `TopicQos`**:

```text
.from_ros(ros_name, TopicQos).to_bus(bus_name, TopicQos).mapper(...).lazy()?.add()
.from_bus(bus_name, TopicQos).to_ros(ros_name, TopicQos).mapper(...).add()
.service().from_ros(ros_name, TopicQos).to_bus(bus_name, TopicQos).mapper(...).timeout()?.add()
.service().from_bus(bus_name, TopicQos).to_ros(ros_name, TopicQos).mapper(...).timeout()?.add()
.action().from_ros(ros_name, TopicQos).to_bus(bus_name, TopicQos).mapper(...).timeout()?.add()
.action().from_bus(bus_name, TopicQos).to_ros(ros_name, TopicQos).mapper(...).timeout()?.add()
```

Prefer named `TopicQos` presets (`default()`, `sensor_data()`, `latched()`, `bus()`; C++ uses `ros_default()` because `default` is a keyword). Custom depth still uses `keep_last(n).reliable()` / `.best_effort()`. ROS endpoints accept either reliability; **bus** endpoints (topic, service, action) must be `.best_effort()` (no DDS reliability) — so `default()` / `latched()` are rejected on bus. Direction is the `from_ros → to_bus` / `from_bus → to_ros` chain. For services and actions this is the **request / goal direction**: `from_ros → to_bus` exposes a ROS proxy server backed by a bus server; `from_bus → to_ros` exposes a bus proxy server backed by a ROS server. Responses, feedback, and results travel back to the caller. **`both` is not allowed**. Typical ROS service QoS (matching `services_default`) is `TopicQos.default()`. Typical bus RPC QoS is `TopicQos.bus()` (depth → DEALER HWM). Action applies the ROS profile to goal / result / cancel services and the feedback topic; the status topic stays the ROS action-status default.

Mount several topics on one bridge by `.add()` then starting the next route. A second topic is another `.from_ros` / `.from_bus`, **not** `.service()` / `.action()` (those switch kind). Directions may mix.

```python
.from_ros("/chatter", TopicQos.default())
.to_bus("/chatter", TopicQos.bus())
.mapper(StdMsgsStringMapper())
.add()
.from_ros("/pose", TopicQos.default())
.to_bus("/pose", TopicQos.bus())
.mapper(GeometryMsgsPoseStampedMapper())
.add()
```

```rust
.from_ros("/chatter", TopicQos::default())
.to_bus("/chatter", TopicQos::bus())
.mapper(StdMsgsStringMapper)
.add()?
.from_ros("/pose", TopicQos::default())
.to_bus("/pose", TopicQos::bus())
.mapper(GeometryMsgsPoseStampedMapper)
.add()?
```

```cpp
.from_ros("/chatter", robot_bus::TopicQos::ros_default())
.to_bus("/chatter", robot_bus::TopicQos::bus())
.mapper(robot_bus::StdMsgsStringMapper{})
.add()
.from_ros("/pose", robot_bus::TopicQos::ros_default())
.to_bus("/pose", robot_bus::TopicQos::bus())
.mapper(robot_bus::GeometryMsgsPoseStampedMapper{})
.add()
```

- Default timeouts: service **5s**, action goal **30s**
- Topic routes default to **eager** ROS subscriptions (`build()` creates them immediately so the ROS graph shows the bridge). Opt in to on-demand ROS2→bus with `.lazy()` on that route only.

### `.lazy()` (opt-in ROS2→bus)

Default is eager: `.from_ros(...).to_bus(...).mapper(...).add()` creates the ROS subscription at `build()`. Use `.lazy()` only on high-bandwidth ROS2→bus topics (camera, lidar) so the ROS graph has no bridge subscriber until a robot-bus subscriber exists.

```rust
.from_ros("/camera/image", TopicQos::sensor_data())
.to_bus("/camera/image", TopicQos::sensor_data())
.mapper(SensorMsgsImageMapper)
.lazy()
.add()?
```

```python
.from_ros("/camera/image", TopicQos.sensor_data())
.to_bus("/camera/image", TopicQos.sensor_data())
.mapper(SensorMsgsImageMapper())
.lazy()
.add()
```

```cpp
.from_ros("/camera/image", robot_bus::TopicQos::sensor_data())
.to_bus("/camera/image", robot_bus::TopicQos::sensor_data())
.mapper(robot_bus::SensorMsgsImageMapper{})
.lazy()
.add()
```

Rules:

- **Default eager.** Omit `.lazy()`.
- **No-arg.** `.lazy()` only; not `.lazy(true)`.
- **Only on `from_ros → to_bus`.** `from_bus → to_ros` has no `.lazy()`. Service / action builders have no `.lazy()`.
- **No-console broker** (`--no-console`): `.lazy()` routes **fall back to eager** (there is no demand signal).
- Demand counts `kind == subscriber` on the bus topic. A raw `Subscriber` (no `Node`) and a WebSocket client with topology off do **not** open a lazy route. After a crash, topology TTL is about 30s.

The broker publishes immediate [`TopicDemand`](../../proto/robot_bus_interfaces/msg/v1/console_status.proto) on `/robot_bus/topic_demand` when a subscriber registers or unregisters. Bridges also read `/robot_bus/topics` so a late-starting lazy route still sees existing subscribers.

C++ custom mappers that only override `attach` (entities stuffed into `keep_alive`) cannot tear the ROS subscription down; `.lazy().add()` throws. Use `TypedTopicMapper`.

### TopicQos

Same type in all three languages. Pass one copy per endpoint (depth / reliability may differ). Usual routes use named presets; custom depth still uses `keep_last(n).reliable()` / `.best_effort()`.

| Preset | Value | Matches / use |
|--|--|--|
| `TopicQos.default()` (C++: `ros_default()`) | KeepLast 10, reliable, volatile | Ordinary topics, services, actions. ROS `qos_profile_default` / `ServicesQoS`. **Not** ROS `SystemDefaultsQoS`. |
| `TopicQos.sensor_data()` | KeepLast 5, best effort, volatile | Camera, lidar, IMU. ROS `SensorDataQoS`. |
| `TopicQos.latched()` | KeepLast 1, reliable, transient local | `/tf_static` and other latched ROS topics. |
| `TopicQos.bus()` | KeepLast 8, best effort | All bus endpoints (depth → HWM). |

- **ROS** (`from_ros` / `to_ros`): preset or `keep_last`. Same `TopicQos` on topic, service, and action ROS endpoints. C++ uses `.durability_volatile()` to unset latch (`volatile` is a keyword).
- **bus** (`from_bus` / `to_bus`): depth → HWM only (topic PUB/SUB, service/action DEALER); must be `.best_effort()` (`sensor_data()` / `bus()` / custom). Durability is ignored.

You can mix a preset on one end and `keep_last` on the other.

### Built-in mappers (core distro only)

**Topic mappers share one catalog across Rust / Python / C++** (~125 types from Humble/Jazzy common interface packages): Rust `src/ros2_bridge/mappers/`, Python `from robot_bus.ros2_bridge import GeometryMsgsPoseStampedMapper`, C++ `#include <robot_bus/ros2_bridge.hpp>` (umbrella `ros2_bridge_topic_mappers.hpp`). Package allowlist: `CORE_BRIDGE_PACKAGES` in `scripts/generate_topic_mappers.py`. After changing protos, re-run `just gen-topic-mappers`. Mount with `.mapper(GeometryMsgsPoseStampedMapper())` (Python) / `.mapper(GeometryMsgsPoseStampedMapper{})` (C++; same attach path as custom mappers; `TopicBuiltin` stays String/Image only).

**Not bridge builtins:** `nav2_msgs` / `control_msgs` / `foxglove_msgs` / `apriltag_msgs`. Matching **protos may remain** for bus-native APIs; ROS↔bus conversion requires a user TypedTopicMapper. Installing those apt packages later does **not** enable bridge mappers automatically.

Service / action remain hand-written builtins (no generated srv/action catalogs):

| Kind | Mapper | ROS type |
|------|--------|----------|
| Topic (example) | `StdMsgsStringMapper` | `std_msgs/msg/String` |
| Topic (example) | `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| Topic (example) | `GeometryMsgsPoseStampedMapper` | `geometry_msgs/msg/PoseStamped` |
| Service | `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| Service | `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |
| Action | `FibonacciActionMapper` | `example_interfaces/action/Fibonacci` |

Rust also exposes `lookup_topic_mapper` / `registered_topic_types` for the core set only.

**In Python you can pass the ROS type name** instead of the mapper class, for topics, services and actions. The `msg` / `srv` / `action` segment is optional and the type name is case-insensitive. A typo gets an error listing close matches.

```python
.from_ros("/imu", TopicQos.sensor_data()).to_bus("/imu", TopicQos.bus())
.mapper("sensor_msgs/msg/Imu").add()
.service().from_bus("/reset", TopicQos.bus()).to_ros("/reset", TopicQos.default())
.mapper("std_srvs/srv/Trigger").add()
```

Custom types work the same way after a one-line registration; see "Python: custom Service mapper" below. Passing a mapper object still works.

---

## User-defined mappers

First write a **bus protobuf** (fields aligned with the ROS `.msg` / `.srv` / `.action`), generate language stubs with `protoc`, then only write **field ↔ protobuf conversion**; the library handles subscribe/publish/service wiring. Typed APIs accept any protobuf message class — they do not have to live in this repository.

| Language | How |
|--|--------|
| Python | duck-typed convert methods + `.mapper(MyFoo())` |
| Rust | `impl TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` |
| C++ | `TypedTopicMapper` / `TypedServiceMapper` CRTP + `.mapper(M{})` or `.mapper(shared_ptr)` (field conversion requires `ROBOT_BUS_HAS_ROS2`) |

Advanced: you can still override `ServiceMapper::attach` / `ActionMapper::attach` directly (special QoS, etc.).

Below uses a custom ROS `example_interfaces/srv/AddTwoInts` (same shape as a
project-local `my_pkg/srv/AddTwoInts`), starting from the proto through mounting
the bridge.

**Runnable demos** (Python / Rust / C++): [`examples/ros2_bridge/`](../../examples/ros2_bridge/)
— `builtin` for built-in mappers, `custom_add_two_ints` for this custom flow.

### 1. Define the bus protobuf

ROS side already exists (`example_interfaces` on Humble/Jazzy):

```text
# example_interfaces/srv/AddTwoInts.srv
int64 a
int64 b
---
int64 sum
```

On the bus, write a `.proto` with the same fields (ROS-style package path + `v1`
recommended). This repo already ships
[`proto/example_interfaces/srv/v1/add_two_ints.proto`](../../proto/example_interfaces/srv/v1/add_two_ints.proto):

```protobuf
syntax = "proto3";
package example_interfaces.srv.v1;

// Equivalent to ROS 2 `example_interfaces/srv/AddTwoInts`.
message AddTwoIntsRequest {
  int64 a = 1;
  int64 b = 2;
}

message AddTwoIntsResponse {
  int64 sum = 1;
}
```

For a **project-local** type (e.g. `my_pkg`), generate stubs yourself:

```bash
# Python
protoc --python_out=. --pyi_out=. my_pkg/srv/v1/add_two_ints.proto

# C++
protoc --cpp_out=. my_pkg/srv/v1/add_two_ints.proto
```

Rust, in `build.rs`:

```rust
prost_build::compile_protos(
    &["proto/my_pkg/srv/v1/add_two_ints.proto"],
    &["proto"],
)?;
```

To contribute a type into this repo’s built-in set: add the file under [`proto/`](../../proto/) and regenerate with `just gen-*`.

### Python: custom Service mapper

The bridge calls: `ros_srv_type()`, `ros_req_to_bus` / `bus_req_to_ros`, `ros_resp_to_bus` / `bus_resp_to_ros`.

Full runnable file: [`examples/ros2_bridge/python/custom_add_two_ints.py`](../../examples/ros2_bridge/python/custom_add_two_ints.py).

```python
from example_interfaces.srv import AddTwoInts
from robot_bus.example_interfaces.srv.v1 import add_two_ints_pb2 as pb
from robot_bus.ros2_bridge import Direction, Ros2Bridge

class AddTwoIntsServiceMapper:
    def type_name(self) -> str:
        return "example_interfaces/srv/AddTwoInts"

    def ros_srv_type(self):
        return AddTwoInts

    def ros_req_to_bus(self, req) -> bytes:
        return pb.AddTwoIntsRequest(a=int(req.a), b=int(req.b)).SerializeToString()

    def bus_req_to_ros(self, payload: bytes):
        bus = pb.AddTwoIntsRequest()
        bus.ParseFromString(payload)
        out = AddTwoInts.Request()
        out.a = int(bus.a)
        out.b = int(bus.b)
        return out

    def ros_resp_to_bus(self, resp) -> bytes:
        return pb.AddTwoIntsResponse(sum=int(resp.sum)).SerializeToString()

    def bus_resp_to_ros(self, payload: bytes):
        bus = pb.AddTwoIntsResponse()
        bus.ParseFromString(payload)
        out = AddTwoInts.Response()
        out.sum = int(bus.sum)
        return out

bridge = (
    Ros2Bridge.new("bridge")
    .bus_tcp("localhost")
    .service()
    .from_ros("/examples/add_two_ints", TopicQos.default())
    .to_bus("/examples/add_two_ints", TopicQos.bus())
    .mapper(AddTwoIntsServiceMapper())
    .timeout(5.0)
    .add()
    .build()
)
```

Action: same flow — three proto messages (Goal / Feedback / Result), then `ros_action_type()` + six-way conversion (see [`mappers/fibonacci.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/fibonacci.py)).

To refer to a custom mapper by type name elsewhere (or from a future config file), register it once. This works the same for topic, service and action mappers:

```python
from robot_bus.ros2_bridge import register_mapper

register_mapper("example_interfaces/srv/AddTwoInts", AddTwoIntsServiceMapper)

.service().from_ros("/examples/add_two_ints", TopicQos.default())
.to_bus("/examples/add_two_ints", TopicQos.bus())
.mapper("example_interfaces/srv/AddTwoInts").add()
```

`@register_mapper("my_pkg/msg/Sum")` also works as a class decorator. Registering a name that already exists, built-in or not, raises an error; pass `override=True` to replace it on purpose.

### Rust: custom Service (`TypedServiceMapper`)

After `include!` of the generated code, encode/decode with prost types.
Runnable: [`examples/ros2_bridge/rust/custom_add_two_ints.rs`](../../examples/ros2_bridge/rust/custom_add_two_ints.rs).

```rust
use prost::Message as ProstMessage;
use ros_env::example_interfaces::srv as ros_srv;
use robot_bus::example_interfaces::srv::v1::{AddTwoIntsRequest, AddTwoIntsResponse};
use robot_bus::ros2_bridge::TypedServiceMapper;

#[derive(Clone, Copy)]
struct AddTwoIntsServiceMapper;

impl TypedServiceMapper for AddTwoIntsServiceMapper {
    type Ros = ros_srv::AddTwoInts;

    fn type_name(&self) -> &str {
        "example_interfaces/srv/AddTwoInts"
    }

    fn ros_req_to_bus(&self, req: &ros_srv::AddTwoInts_Request) -> robot_bus::Result<Vec<u8>> {
        Ok(AddTwoIntsRequest { a: req.a, b: req.b }.encode_to_vec())
    }

    fn bus_req_to_ros(&self, payload: &[u8]) -> robot_bus::Result<ros_srv::AddTwoInts_Request> {
        let bus = AddTwoIntsRequest::decode(payload)?;
        Ok(ros_srv::AddTwoInts_Request { a: bus.a, b: bus.b })
    }

    fn ros_resp_to_bus(&self, resp: &ros_srv::AddTwoInts_Response) -> robot_bus::Result<Vec<u8>> {
        Ok(AddTwoIntsResponse { sum: resp.sum }.encode_to_vec())
    }

    fn bus_resp_to_ros(&self, payload: &[u8]) -> robot_bus::Result<ros_srv::AddTwoInts_Response> {
        let bus = AddTwoIntsResponse::decode(payload)?;
        Ok(ros_srv::AddTwoInts_Response { sum: bus.sum })
    }
}

// .service().from_ros("/examples/add_two_ints", TopicQos::default()).to_bus("/examples/add_two_ints", TopicQos::bus())
//     .mapper(AddTwoIntsServiceMapper)
//     .add()?
```

Action: `impl TypedActionMapper` (`type Ros = …` + six-way goal/feedback/result conversion). The library's `wire_typed_*` handles wiring.

Custom topic: `impl TypedTopicMapper` (associated `Ros` IDL type and `Bus` protobuf type), conversion only; the library owns `create_subscription` / `create_publisher`.

```rust
use robot_bus::ros2_bridge::TypedTopicMapper;

#[derive(Clone, Copy)]
struct MyStringMapper;

impl TypedTopicMapper for MyStringMapper {
    type Ros = ros_env::std_msgs::msg::String;
    type Bus = robot_bus::std_msgs::msg::v1::String;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::Result<Self::Bus> {
        Ok(Self::Bus { data: msg.data.to_string() })
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::Result<Self::Ros> {
        Ok(Self::Ros { data: msg.data.into() })
    }
}
```

### C++: custom Service (`TypedServiceMapper` CRTP)

Built-ins use ZSTs: `.mapper(TriggerServiceMapper{})`. Custom mappers inherit CRTP, implement conversion only; the library auto `attach` / `retain`.
Runnable: [`examples/ros2_bridge/cpp/custom_add_two_ints.cpp`](../../examples/ros2_bridge/cpp/custom_add_two_ints.cpp).

```cpp
#include <robot_bus/ros2_bridge.hpp>
#include <example_interfaces/srv/add_two_ints.hpp>
#include <robot_bus/example_interfaces/srv/v1/add_two_ints.pb.h>

struct AddTwoIntsServiceMapper
    : robot_bus::TypedServiceMapper<AddTwoIntsServiceMapper,
                                    example_interfaces::srv::AddTwoInts> {
  const char *type_name() const override {
    return "example_interfaces/srv/AddTwoInts";
  }

  std::vector<uint8_t> ros_req_to_bus(const Request &req) const {
    example_interfaces::srv::v1::AddTwoIntsRequest bus;
    bus.set_a(req.a);
    bus.set_b(req.b);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return {bytes.begin(), bytes.end()};
  }

  Request bus_req_to_ros(robot_bus::BytesView body) const {
    example_interfaces::srv::v1::AddTwoIntsRequest bus;
    bus.ParseFromArray(body.data, static_cast<int>(body.size));
    Request out;
    out.a = bus.a();
    out.b = bus.b();
    return out;
  }

  std::vector<uint8_t> ros_resp_to_bus(const Response &resp) const {
    example_interfaces::srv::v1::AddTwoIntsResponse bus;
    bus.set_sum(resp.sum);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return {bytes.begin(), bytes.end()};
  }

  Response bus_resp_to_ros(robot_bus::BytesView body) const {
    example_interfaces::srv::v1::AddTwoIntsResponse bus;
    bus.ParseFromArray(body.data, static_cast<int>(body.size));
    Response out;
    out.sum = bus.sum();
    return out;
  }
};

// .service().from_ros("/examples/add_two_ints", robot_bus::TopicQos::ros_default())
//     .to_bus("/examples/add_two_ints", robot_bus::TopicQos::bus())
//     .mapper(std::make_shared<AddTwoIntsServiceMapper>())
//     .add()
```

Topic / Action: same “proto first, then mapper” flow. `TypedTopicMapper` / `TypedActionMapper` (see [`ros2_bridge_typed.hpp`](../../bindings/cpp/include/robot_bus/ros2_bridge_typed.hpp)).

---

## Rust (`rclrs`)

```rust
use robot_bus::ros2_bridge::{
    Ros2Bridge, StdMsgsStringMapper, TopicQos, TriggerServiceMapper,
};

fn main() -> robot_bus::Result<()> {
    let mut bridge = Ros2Bridge::new("ros_bridge")
        .bus_tcp("localhost")
        .from_ros("/chatter", TopicQos::default())
            .to_bus("/chatter", TopicQos::bus())
            .mapper(StdMsgsStringMapper)
            .add()?
        .service()
            .from_ros("/reset", TopicQos::default())
            .to_bus("/reset", TopicQos::bus())
            .mapper(TriggerServiceMapper)
            .timeout(std::time::Duration::from_secs(3))
            .add()?
        .build()?;
    bridge.spin()?;
    Ok(())
}
```

- Custom topic: `impl TypedTopicMapper` (associated `Ros` / `Bus`, field-wise `ros_to_bus` / `bus_to_ros`; the library wires subscribe/publish)
- Custom service/action: `TypedServiceMapper` / `TypedActionMapper` (see "User-defined" above)
- Modules: `typed_service` (`wire_typed_*` / `attach_*`)

### Rust messages (`ros-env` + ament rust IDL)

Cargo builds the bridge with **`rclrs` 0.8**, **`ros-env` 0.3**, and **`rosidl_runtime_rs` 0.7**. `ros-env` reads the generated Rust bindings from the sourced ROS environment.

The `ros2` Cargo feature requires generated Rust interfaces (`share/<pkg>/rust/` under `AMENT_PREFIX_PATH`) and their ROS system libraries. Use custom mappers for extension packages such as Nav2 and Foxglove.

With ROS 2 Humble and the build tools installed (Rust, Protobuf compiler 35.1, C/C++ toolchain and pkg-config), run from the repository root:

```bash
sudo apt-get install libzmq3-dev ros-humble-rosidl-generator-rs \
  ros-humble-example-interfaces ros-humble-test-msgs
source /opt/ros/humble/setup.bash
python3 scripts/generate_rust_msgs.py
cargo build --no-default-features --features ros2 --bin ros2_bridge_perf \
  --example ros2_bridge_builtin --example ros2_bridge_custom_add_two_ints
ROS_LOCALHOST_ONLY=1 ROS_DOMAIN_ID=94 RMW_IMPLEMENTATION=rmw_fastrtps_cpp \
  cargo test --no-default-features --features ws,console-api,ros2 \
  --test ros2_bridge_lazy --test ros2_bridge_service_smoke \
  --test ros2_bridge_diagnostics -- --test-threads=1
```

Use interface packages that ship compatible Rust bindings. On Jazzy, install the corresponding `ros-jazzy-*` packages, source `/opt/ros/jazzy/setup.bash`, and use `ROS_AUTOMATIC_DISCOVERY_RANGE=LOCALHOST` for local tests. `example_interfaces` provides Fibonacci and AddTwoInts. See [support and verification status](support.md) for the tested environments.

---

## Python (`rclpy`)

Implementation directory: [`bindings/python/robot_bus/ros2_bridge/`](../../bindings/python/robot_bus/ros2_bridge/) (pure Python).

```bash
source /opt/ros/humble/setup.bash
just python-dev-ros2   # or just python-dev; requires local rclpy
```

```python
import robot_bus
from robot_bus.ros2_bridge import (
    Ros2Bridge,
    StdMsgsStringMapper,
    TopicQos,
    TriggerServiceMapper,
)

assert robot_bus.ros2_available()  # import rclpy succeeds

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", TopicQos.default())
    .to_bus("/chatter", TopicQos.bus())
    .mapper(StdMsgsStringMapper())
    .add()
    .service()
    .from_ros("/reset", TopicQos.default())
    .to_bus("/reset", TopicQos.bus())
    .mapper(TriggerServiceMapper())
    .add()
    .build()
)
bridge.run()  # Ctrl+C / SIGTERM to stop
```

- ROS side: `rclpy` node + executor (background thread spin)
- Bus side: `robot_bus.Node` (raw / typed protobuf)
- **Threading:** `ServiceClient` / `TopicPublisher` / `ActionClient` are process-safe (`Send + Sync` via a per-handle mutex around the ZMQ socket). `Ros2ToBus` callbacks on the rclpy executor thread may call them directly (same idea as C++ `std::mutex` around the client). Concurrent calls on one handle are serialised.
- Custom mappers: see "User-defined" above; built-in reference [`mappers/trigger.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/trigger.py)
- Mappers lazy-import on demand (depends on corresponding ROS message packages and protobuf)

---

## C++ (`rclcpp`)

```cpp
#include <robot_bus/ros2_bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", robot_bus::TopicQos::ros_default())
    .to_bus("/chatter", robot_bus::TopicQos::bus())
    .mapper(robot_bus::StdMsgsStringMapper{})
    .add()
    .service()
    .from_ros("/reset", robot_bus::TopicQos::ros_default())
    .to_bus("/reset", robot_bus::TopicQos::bus())
    .mapper(robot_bus::TriggerServiceMapper{})
    .add()
    .build();

bridge.spin();
```

- Link **`robot_bus_ros2_bridge`** (`ROBOT_BUS_HAS_ROS2`); without this macro, `build()` throws
- Local build: `just cpp-dev-ros2` (requires `just gen-cpp` + source ROS first)
- Packages: `robot-bus-ros2-humble` / `robot-bus-ros2-jazzy` (**does not** vendor `rcl`)
- Built-in ZSTs + `.mapper(std::shared_ptr<…Mapper>)` for custom; see "User-defined" above

---

## Runtime

Same process holds both:

1. ROS node (rclrs / rclpy / rclcpp)
2. robot-bus `Node`

The main loop must drive both sides (`spin` / `spin_once`); implementation details differ per language, semantics are the same: drain ROS↔bus queues and drive the bus.

Python: `bridge.run()` spins until Ctrl+C, SIGTERM or ROS shutdown, then calls `bridge.shutdown()`. With your own loop (for example, also driving another bus `Node`), use `with bridge:` so it shuts down on exit. `shutdown()` is idempotent; `close()` is an alias.

```python
with bridge:
    while True:
        my_node.spin_once(0.01)
        bridge.spin_once(0.01)
```

Topic convert / decode / publish failures are dropped (the bridge stays up). Each bridge keeps atomic counters; `drop_stats()` returns the **sum** across routes (`convert_fail` / `decode_fail` / `publish_fail`). Failure logs are rate-limited per route (first plus at most one per second). `build()` prints the route table. Per-topic route counters are published at 1 Hz on `/robot_bus/bridges` (console **BRIDGE** tab). After 15s without topic traffic the bridge warns once per quiet episode on `/robot_bus/events`; receiving traffic rearms the warning. Transient-local topics are exempt after their first sample. See the diagnostics section for counters and interpretation.

```python
bridge.drop_stats()  # {"convert_fail": 0, "decode_fail": 0, "publish_fail": 0}
```

```rust
let snap = bridge.drop_stats(); // snap.convert_fail / decode_fail / publish_fail
```

```cpp
auto snap = bridge.drop_stats();  // snap.convert_fail / decode_fail / publish_fail
```

---

## FAQ

1. **ROS not sourced** — all three language bindings fail.
2. **C++ `ros2_available() == false`** — not linked with `robot_bus_ros2_bridge` / installed package without bridge.
3. **Python `ros2_available() == False`** — `rclpy` not installed or ROS not sourced.
4. **Rust topic registered but fails at runtime** — missing corresponding ROS typesupport, or rust IDL not on `AMENT_PREFIX_PATH`.

---

## Related

- C++ packages and local build: [cpp-api.md](cpp-api.md)
- Python SDK: [python-api.md](python-api.md)
- API comparison: [api-compare.md](api-compare.md)


## Terminal outcomes and diagnostics

For bus clients calling ROS actions, only SUCCEEDED returns the mapped result.
Rejected goals, ABORTED, CANCELED, timeouts and bridge exceptions become distinct
bus errors: Rust `ActionRejected`, `ActionAborted`, `Cancelled`, `Timeout` and
`Protocol`; Python/C++ receive exceptions with the corresponding messages.
WebSocket statuses are 9, 10, 1, 4 and 13 respectively. Result timeout requests ROS
cancellation on a best-effort basis; this does not confirm that execution stopped.
For ROS clients calling bus actions, success/cancel/failure map to
SUCCEEDED/CANCELED/ABORTED. In C++, `rclcpp` requires the goal to be cancelling
before reporting CANCELED. An unsolicited bus cancellation without a ROS cancel
request therefore ends as ABORTED, while diagnostics retain the cancellation.
A downstream rejection after the bridge has accepted
the ROS goal ends as ABORTED, with the rejection retained in bridge diagnostics.
Error outcomes do not carry a partial result payload.

The bridge console includes RPC calls, successful completions, active calls,
failures (including timeout/rejection subtotals), cancellations, last terminal
status and last error. These measure bridge/transport outcomes, not application
fields such as `success=false`. Last error is retained after later success.
Builtin and Typed mappers record them automatically; custom `attach` implementations
can use `ctx.route_health` (Rust) or `ctx.health` (Python/C++) to record start/finish.


**When a ROS client calls a bus service and the bridge call fails (`.from_ros → .to_bus`):** the ROS service protocol has no error channel, only response fields.

All three languages behave the same way:

- If the mapper provides `error_response(message)` (built-in `Trigger` / `SetBool` do, returning `success=False` plus the message), the bridge replies with it.
- Otherwise the bridge sends **no reply**, so the ROS caller times out instead of reading a default-valued response as a real result.
- Both cases log a rate-limited message with the service name and reason, and the console failure counters still increase.

To reply for a custom service, implement `error_response` for a response type that has a way to signal failure:

| Language | Signature |
| --- | --- |
| Python | `def error_response(self, message): ...` returns a ROS response |
| Rust | `fn error_response(&self, message: &str) -> Option<Response>` (the default returns `None`) |
| C++ | `std::optional<Response> error_response(const std::string &message) const` (the default returns `std::nullopt`; returning a plain `Response` also works) |

The Rust bridge keeps each unanswered service callback pending until shutdown. This is a small per-failure leak, which is why `error_response` is worth implementing for services that can fail often.

Topic warnings now detect both missing first traffic after 15 seconds and a
15-second gap after receiving data. One warning is emitted per quiet episode;
new traffic rearms it. Disabled lazy routes are exempt, and transient-local
static topics stop being watched after their first sample. Quiet event/low-rate
topics may be healthy; inspect the source before treating a warning as a fault.

Upgrade the bridge, client SDK and WebSocket server together for the new
`ACTION_REJECTED`, `ACTION_ABORTED`, `RPC_TIMEOUT` and `RPC_FAILED` error markers.
Successful result payloads are unchanged; cancellation uses existing `CANCELLED`.
Older SDKs cannot classify the new errors.

See [support status](support.md) for live ROS service and action coverage by language.
