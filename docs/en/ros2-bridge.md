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

**Why per language:** Topic / service / action all need compile-time concrete types (`create_subscription<T>`, `create_service<T>`, etc.). If C++/Python only pass type names to Rust, `T` won't match. Each language therefore creates ROS entities with concrete types on its side, then forwards via its own bus `Node`.

| Supported | Not supported |
|-----------|---------------|
| Topic / Service / Action | YAML-configured bridge |
| `.mapper(concrete object)` in code | Mounting routes via type-name string lookup |
| User-defined mappers (concrete types per language) | Cross-language "string-only" universal bridge |

Official releases: **Humble**, **Jazzy**.

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

## Unified contract (code only)

Direction (`Direction`): `Ros2ToBus` (default) or `BusToRos2`; **`both` is not allowed**.

```text
Ros2Bridge.new / New / new(name)
  .bus_tcp(...) | .bus_ipc() | .bus_discover(...)
  .route(ros, bus).mapper(...).direction(...).qos_depth(n)|.best_effort()|.sensor_data().lazy().add()
  .service(ros, bus).mapper(...).timeout(...).direction(...).add()
  .action(ros, bus).mapper(...).timeout(...).direction(...).add()
  .build()
  .spin() | .spin_once(...)
```

- Default timeouts: service **5s**, action goal **30s**
- **No** `from_yaml`; **no** `add_route(..., "pkg/msg/Type", ...)`
- Topic routes default to **eager** ROS subscriptions (`build()` creates them immediately so the ROS graph shows the bridge). Opt in to on-demand ROS2→bus with `.lazy()` on that route only.

### `.lazy()` (opt-in ROS2→bus)

Default matches 1.3.1: `.route(...).mapper(...).add()` creates the ROS subscription at `build()`. Use `.lazy()` only on high-bandwidth ROS2→bus topics (camera, lidar) so the ROS graph has no bridge subscriber until a robot-bus subscriber exists.

```rust
.route("/camera/image", "/camera/image")
    .mapper(SensorMsgsImageMapper)
    .sensor_data()
    .lazy()
    .add()?
```

```python
.route("/camera/image", "/camera/image")
    .mapper(SensorMsgsImageMapper())
    .sensor_data()
    .lazy()
    .add()
```

```cpp
.route("/camera/image", "/camera/image")
    .mapper(robot_bus::SensorMsgsImageMapper{})
    .sensor_data()
    .lazy()
    .add()
```

Rules:

- **Default eager.** Existing examples do not need `.lazy()`.
- **No-arg.** `.lazy()` only; not `.lazy(true)`, not a new `Direction`.
- **ROS2→bus only.** `.lazy()` on `BusToRos2` fails at `.add()`. Service / action builders have no `.lazy()`.
- **No-console broker** (`--no-console`): `.lazy()` routes **fall back to eager** (there is no demand signal).
- Demand counts `kind == subscriber` on the bus topic. A raw `Subscriber` (no `Node`) and a WebSocket client with topology off do **not** open a lazy route. After a crash, topology TTL is about 30s.

The broker publishes immediate [`TopicDemand`](../../proto/robot_bus_interfaces/msg/v1/console_status.proto) on `/robot_bus/topic_demand` when a subscriber registers or unregisters. Bridges also read `/robot_bus/topics` so a late-starting lazy route still sees existing subscribers.

C++ custom mappers that only override `attach` (entities stuffed into `keep_alive`) cannot tear the ROS subscription down; `.lazy().add()` throws. Use `TypedTopicMapper`.

### Topic route QoS helpers (opt-in)

Defaults are unchanged: C++ / Python `QoS(10)` reliable; Rust `topics_default()`. Helpers apply to **topics only** (not service / action):

| helper | ROS | bus |
|--------|-----|-----|
| `.qos_depth(n)` | KeepLast(n) | `QosProfile::keep_last(n)` |
| `.best_effort()` | reliability = best effort | (bus HWM is already best-effort) |
| `.sensor_data()` | `SensorDataQoS` (best-effort KeepLast 5) | depth 5 |

Camera example: `.sensor_data().lazy()`. Image builtins do **not** default to SensorDataQoS.

### Phase-1 built-in mappers (objects, not strings)

| Kind | Mapper | ROS type |
|------|--------|----------|
| Topic | `StdMsgsStringMapper` | `std_msgs/msg/String` |
| Topic | `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| Service | `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| Service | `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |
| Action | `FibonacciActionMapper` | `example_interfaces/action/Fibonacci` |

Rust also has a full topic mapper registry (`src/ros2_bridge/mappers/`); mounting routes still requires `.mapper(concrete type)`; `lookup_topic_mapper` / `registered_topic_types` are for introspection only, not for mounting routes.

---

## User-defined mappers: yes

**Yes.** First write a **bus protobuf** (fields aligned with the ROS `.msg` / `.srv` / `.action`), generate language stubs with `protoc`, then only write **field ↔ protobuf conversion**; the library handles subscribe/publish/service wiring. Typed APIs accept any protobuf message class — they do not have to live in this repository.

| | Works? |
|--|--------|
| Python: duck-typed convert methods + `.mapper(MyFoo())` | **Yes** |
| Rust: `impl TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` | **Yes** |
| C++: `TypedTopicMapper` / `TypedServiceMapper` CRTP + `.mapper(shared_ptr)` | **Yes** (requires `ROBOT_BUS_HAS_ROS2`) |
| YAML / type-name strings only | **No** |

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
    .service("/examples/add_two_ints", "/examples/add_two_ints")
    .mapper(AddTwoIntsServiceMapper())
    .direction(Direction.Ros2ToBus)
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

// .service("/examples/add_two_ints", "/examples/add_two_ints")
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

// .service("/examples/add_two_ints", "/examples/add_two_ints")
//     .mapper(std::make_shared<AddTwoIntsServiceMapper>())
//     .direction(robot_bus::Direction::Ros2ToBus)
//     .add()
```

Topic / Action: same “proto first, then mapper” flow. `TypedTopicMapper` / `TypedActionMapper` (see [`ros2_bridge_typed.hpp`](../../bindings/cpp/include/robot_bus/ros2_bridge_typed.hpp)).

---

## Rust (`rclrs`)

```rust
use robot_bus::ros2_bridge::{
    Direction, Ros2Bridge, StdMsgsStringMapper, TriggerServiceMapper,
};

fn main() -> robot_bus::Result<()> {
    let mut bridge = Ros2Bridge::new("ros_bridge")
        .bus_tcp("localhost")
        .route("/chatter", "/chatter")
            .mapper(StdMsgsStringMapper)
            .direction(Direction::Ros2ToBus)
            .add()?
        .service("/reset", "/reset")
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
    Direction,
    Ros2Bridge,
    StdMsgsStringMapper,
    TriggerServiceMapper,
)

assert robot_bus.ros2_available()  # import rclpy succeeds

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
    .route("/chatter", "/chatter")
    .mapper(robot_bus::StdMsgsStringMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .service("/reset", "/reset")
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
2. **YAML bridge config** — not supported; mount mappers in code.
3. **Type-name strings only** — not supported for mounting routes; pass concrete mapper objects.
4. **Cross-language universal dynamic srv** — not supported; write a custom mapper in the target language.
5. **C++ `ros2_available() == false`** — not linked with `robot_bus_ros2_bridge` / installed package without bridge.
6. **Python `ros2_available() == False`** — `rclpy` not installed or ROS not sourced.
7. **Rust topic registered but fails at runtime** — missing corresponding ROS typesupport (e.g. `foxglove_msgs`).

---

## Related

- C++ packages and local build: [cpp-api.md](cpp-api.md)
- Python SDK: [python-api.md](python-api.md)
- API comparison: [api-compare.md](api-compare.md)
