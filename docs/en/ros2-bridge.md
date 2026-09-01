English | [中文](../zh/ros2-bridge.md)

# ROS 2 Bridge (`ros2_bridge`)

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

Official releases: **Humble**, **Jazzy**. Mount routes with `.mapper(concrete object)`; custom mappers are field converters in that language.

---

## Prerequisites

```bash
source /opt/ros/humble/setup.bash   # or jazzy
cargo run --bin robot_bus_broker    # or installed robot-bus-broker
```

**Rust `feature = "ros2"`** uses crates.io **`rclrs` 0.7**. Typed messages come from published **`ros-env` 0.2**, which re-exports `share/<pkg>/rust/` on `AMENT_PREFIX_PATH` as `ros_env::sensor_msgs::msg::Image`. Current Humble apt packages for `common_interfaces` (including `sensor_msgs`) **already ship rust IDL**, so sourcing `/opt/ros/humble` is enough for Image / String and similar. The full mapper registry also needs packages Humble does not install by default (`nav2_msgs` / `control_msgs` / `apriltag_msgs`); only those need an overlay. See **Rust messages** below.

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

After `TopicQos.keep_last(n)` you must call `.reliable()` or `.best_effort()`. ROS endpoints accept either; **bus** endpoints (topic, service, action) must be `.best_effort()` (no DDS reliability). ROS durability defaults to volatile; chain `.transient_local()` on the ROS endpoint for latched topics such as `/tf_static`. Direction is the `from_ros → to_bus` / `from_bus → to_ros` chain. **`both` is not allowed**. Typical ROS service QoS (matching `services_default`) is `TopicQos.keep_last(10).reliable()`. Typical bus RPC QoS is `TopicQos.keep_last(8).best_effort()` (depth → DEALER HWM). Action applies the ROS profile to goal / result / cancel services and the feedback topic; the status topic stays the ROS action-status default.

- Default timeouts: service **5s**, action goal **30s**
- Topic routes default to **eager** ROS subscriptions (`build()` creates them immediately so the ROS graph shows the bridge). Opt in to on-demand ROS2→bus with `.lazy()` on that route only.

### `.lazy()` (opt-in ROS2→bus)

Default is eager: `.from_ros(...).to_bus(...).mapper(...).add()` creates the ROS subscription at `build()`. Use `.lazy()` only on high-bandwidth ROS2→bus topics (camera, lidar) so the ROS graph has no bridge subscriber until a robot-bus subscriber exists.

```rust
.from_ros("/camera/image", TopicQos::keep_last(5).best_effort())
.to_bus("/camera/image", TopicQos::keep_last(5).best_effort())
.mapper(SensorMsgsImageMapper)
.lazy()
.add()?
```

```python
.from_ros("/camera/image", TopicQos.keep_last(5).best_effort())
.to_bus("/camera/image", TopicQos.keep_last(5).best_effort())
.mapper(SensorMsgsImageMapper())
.lazy()
.add()
```

```cpp
.from_ros("/camera/image", robot_bus::TopicQos::keep_last(5).best_effort())
.to_bus("/camera/image", robot_bus::TopicQos::keep_last(5).best_effort())
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

Same type in all three languages. Pass one copy per endpoint (depth / reliability may differ):

| | syntax |
|--|--|
| reliable KeepLast 10 | `TopicQos.keep_last(10).reliable()` |
| best-effort KeepLast 5 | `TopicQos.keep_last(5).best_effort()` |
| latched KeepLast 1 (`/tf_static`) | `TopicQos.keep_last(1).reliable().transient_local()` |

- **ROS** (`from_ros` / `to_ros`): KeepLast(depth) + the chosen reliability + durability (default volatile; `.transient_local()` for latch). Same `TopicQos` on topic, service, and action ROS endpoints. C++ uses `.durability_volatile()` to unset latch (`volatile` is a keyword).
- **bus** (`from_bus` / `to_bus`): depth → HWM only (topic PUB/SUB, service/action DEALER); must be `.best_effort()`. Durability is ignored.

To match a ROS graph that uses best-effort KeepLast(5), write `keep_last(5).best_effort()` on both topic ends. For services, `keep_last(10).reliable()` matches ROS `services_default`. For `/tf_static`, write `keep_last(1).reliable().transient_local()` on the ROS end.

### Phase-1 built-in mappers

**Topic mappers share one catalog across Rust / Python / C++** (`proto/*/msg/v1`, ~214 types): Rust `src/ros2_bridge/mappers/`, Python `robot_bus.ros2_bridge.mappers.<pkg>`, C++ `robot_bus/ros2_bridge/mappers/<pkg>/<msg>.hpp`. After changing protos, re-run `just gen-topic-mappers`. Mount with `.mapper(GeometryMsgsPoseStampedMapper())` (Python) / `.mapper(GeometryMsgsPoseStampedMapper{})` (C++; same attach path as custom mappers; `TopicBuiltin` stays String/Image only).

Service / action remain hand-written phase-1 builtins (no generated srv/action catalogs):

| Kind | Mapper | ROS type |
|------|--------|----------|
| Topic (example) | `StdMsgsStringMapper` | `std_msgs/msg/String` |
| Topic (example) | `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| Topic (example) | `GeometryMsgsPoseStampedMapper` | `geometry_msgs/msg/PoseStamped` |
| Service | `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| Service | `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |
| Action | `FibonacciActionMapper` | `example_interfaces/action/Fibonacci` |

Rust also exposes `lookup_topic_mapper` / `registered_topic_types`. C++ treats Humble-optional interface packages (`nav2_msgs` / `control_msgs` / `apriltag_msgs` / `foxglove_msgs`) as `find_package(... QUIET)`: ROS conversion for those mappers compiles only when the package is present.

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
— `builtin` for phase-1 mappers, `custom_add_two_ints` for this custom flow.

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
    .from_ros("/examples/add_two_ints", TopicQos.keep_last(10).reliable())
    .to_bus("/examples/add_two_ints", TopicQos.keep_last(8).best_effort())
    .mapper(AddTwoIntsServiceMapper())
    .timeout(5.0)
    .add()
    .build()
)
```

Action: same flow — three proto messages (Goal / Feedback / Result), then `ros_action_type()` + six-way conversion (see [`mappers/fibonacci.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/fibonacci.py)).

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

// .service().from_ros("/examples/add_two_ints", TopicQos::keep_last(10).reliable()).to_bus("/examples/add_two_ints", TopicQos::keep_last(8).best_effort())
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

    fn type_name(&self) -> &str {
        "std_msgs/msg/String"
    }

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

// .service().from_ros("/examples/add_two_ints", robot_bus::TopicQos::keep_last(10).reliable())
//     .to_bus("/examples/add_two_ints", robot_bus::TopicQos::keep_last(8).best_effort())
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
        .from_ros("/chatter", TopicQos::keep_last(10).reliable())
            .to_bus("/chatter", TopicQos::keep_last(8).best_effort())
            .mapper(StdMsgsStringMapper)
            .add()?
        .service()
            .from_ros("/reset", TopicQos::keep_last(10).reliable())
            .to_bus("/reset", TopicQos::keep_last(8).best_effort())
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

The client is crates.io **`rclrs` 0.7**. Message types come from **`ros-env` 0.2** re-exporting `share/<pkg>/rust/`, not from rclrs itself.

On Humble, `ros-humble-sensor-msgs` and similar packages already include `share/sensor_msgs/rust/` (with `msg::Image`). After `source /opt/ros/humble`, `ros_env` can see those crates. The in-tree topic mapper registry still depends on a few packages **not** in a default apt install (`nav2_msgs`, `control_msgs`, `apriltag_msgs`). For the full registry, put the missing interface packages in an overlay workspace and `colcon build`:

```bash
mkdir -p ~/ros2_rust_ws/src && cd ~/ros2_rust_ws
# Only add packages whose rust IDL is missing from the distro, e.g.:
git clone -b humble https://github.com/ros-navigation/nav2_msgs.git src/nav2_msgs
# likewise control_msgs / apriltag_msgs
git clone https://github.com/ros2-rust/rosidl_rust.git src/rosidl_rust
source /opt/ros/humble/setup.bash
colcon build
source install/setup.bash
# cargo build --features ros2 can then see ros_env::<pkg>::msg
```

Without an overlay, use `just check-ros2-shim`. crates.io `ros-env` empties its shim; this repo patches it with **typed field stubs** in [`third_party/ros-env-shim`](../../third_party/ros-env-shim) (generated from proto; not a DynamicMessage fallback). Our `std_srvs` vendor still uses system C typesupport and does not need rust IDL.

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
    .from_ros("/chatter", TopicQos.keep_last(10).reliable())
    .to_bus("/chatter", TopicQos.keep_last(8).best_effort())
    .mapper(StdMsgsStringMapper())
    .add()
    .service()
    .from_ros("/reset", TopicQos.keep_last(10).reliable())
    .to_bus("/reset", TopicQos.keep_last(8).best_effort())
    .mapper(TriggerServiceMapper())
    .add()
    .build()
)
bridge.spin()
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
    .from_ros("/chatter", robot_bus::TopicQos::keep_last(10).reliable())
    .to_bus("/chatter", robot_bus::TopicQos::keep_last(8).best_effort())
    .mapper(robot_bus::StdMsgsStringMapper{})
    .add()
    .service()
    .from_ros("/reset", robot_bus::TopicQos::keep_last(10).reliable())
    .to_bus("/reset", robot_bus::TopicQos::keep_last(8).best_effort())
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

---

## FAQ

1. **ROS not sourced** — all three language bindings fail.
2. **C++ `ros2_available() == false`** — not linked with `robot_bus_ros2_bridge` / installed package without bridge.
3. **Python `ros2_available() == False`** — `rclpy` not installed or ROS not sourced.
4. **Rust topic registered but fails at runtime** — missing corresponding ROS typesupport (e.g. `foxglove_msgs`).

---

## Related

- C++ packages and local build: [cpp-api.md](cpp-api.md)
- Python SDK: [python-api.md](python-api.md)
- API comparison: [api-compare.md](api-compare.md)
