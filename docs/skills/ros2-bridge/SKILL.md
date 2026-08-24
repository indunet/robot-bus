---
name: ros2-bridge
description: >-
  Create and run robot-bus Ros2Bridge (topic / service / action) between ROS 2
  Humble or Jazzy and robot_bus_broker. Use when the user asks to bridge ROS and
  bus, mount routes with mappers, Direction Ros2ToBus or BusToRos2, custom
  TypedServiceMapper / TypedActionMapper, or connect rclcpp/rclpy/rclrs to bus
  without a full package migration.
---

# Create ROS 2 Bridge (`ros2_bridge`)

In-process bridge: **ROS 2 ↔ robot-bus** for Topic / Service / Action.
Authoritative doc: [ros2-bridge.md](../zh/ros2-bridge.md) ([en](../en/ros2-bridge.md)).

This is **not** full package migration — for that use **ros2-to-robot-bus** /
**robot-bus-to-ros2**. For pure bus pub/sub/service/action/timer use
**robot-bus-primitive**.

## Hard rules

| Do | Do not |
|----|--------|
| Mount routes in **code** with `.mapper(concrete object)` | YAML bridge config |
| One direction per route: `Ros2ToBus` or `BusToRos2` | `both` |
| Implement typed field converters for custom srv/action | Type-name string lookup to mount routes |
| Native bridge per language (rclrs / rclpy / rclcpp) | Cross-language “string-only” universal bridge |

Official ROS targets: **Humble**, **Jazzy**.

## Decide direction

| Data ownership | Direction |
|----------------|-----------|
| ROS publishes / serves → bus clients consume | `Ros2ToBus` (default) |
| Bus publishes / serves → ROS subscribers / clients | `BusToRos2` |

Same logical name on both sides is fine (`/chatter`, `/chatter`); names need not match.

## Prerequisites

```bash
source /opt/ros/humble/setup.bash   # or jazzy
robot-bus-broker                    # or cargo run --bin robot_bus_broker
```

| Language | Needs |
|----------|--------|
| Rust | `robot-bus = { features = ["ros2"] }` |
| Python | `robot_bus` + system `rclpy` (`just python-dev-ros2`) |
| C++ | `robot-bus-ros2-humble` / `…-jazzy` or `just cpp-dev-ros2` (`ROBOT_BUS_HAS_ROS2`) |

Check: `ros2_available()` — Python = `import rclpy`; C++ = linked `robot_bus_ros2_bridge`; Rust FFI ABI always false (bridge not in FFI).

## Workflow checklist

```
Progress:
- [ ] 1. Source ROS; broker reachable (tcp / ipc / discover)
- [ ] 2. Pick language that owns the ROS types
- [ ] 3. List routes: topic/service/action names + direction each
- [ ] 4. Prefer built-in mappers; else define bus `.proto` (fields = ROS .srv/.action), `protoc`, then Typed* mapper
- [ ] 5. Ros2Bridge.new → bus_* → route/service/action → mapper → direction → add → build
- [ ] 6. bridge.spin(); verify ros2 topic echo + bus console
```

## Unified builder contract

```text
Ros2Bridge.new / New / new(name)
  .bus_tcp(...) | .bus_ipc() | .bus_discover(...)
  .route(ros, bus).mapper(...).direction(...).lazy().add()
  .service(ros, bus).mapper(...).timeout(...).direction(...).add()
  .action(ros, bus).mapper(...).timeout(...).direction(...).add()
  .build()
  .spin() | .spin_once(...)
```

Defaults: service timeout **5s**, action goal **30s**. Topic routes are **eager**
at `build()`; `.lazy()` is opt-in ROS2→bus only (camera/lidar). No-console brokers
fall back to eager. `.lazy()` on `BusToRos2` fails at `.add()`.

### Built-in mappers (objects, not strings)

| Kind | Mapper | ROS type |
|------|--------|----------|
| Topic | `StdMsgsStringMapper` | `std_msgs/msg/String` |
| Topic | `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| Service | `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| Service | `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |
| Action | `FibonacciActionMapper` | `example_interfaces/action/Fibonacci` |

Rust may register more topic mappers for introspection; **mounting still requires** `.mapper(concrete type)`.

## Minimal examples

### Python

```python
import robot_bus
from robot_bus.ros2_bridge import (
    Direction, Ros2Bridge, StdMsgsStringMapper, TriggerServiceMapper,
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

### Rust

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

### C++

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

## Custom service / action mappers

Start from a **bus `.proto`** whose fields match the ROS `.srv` / `.action`. Generate stubs with
`protoc` (or `prost-build`), then convert **fields ↔ protobuf**. The library wires
`create_service` / clients. Full listing: [ros2-bridge.md](../zh/ros2-bridge.md).

```protobuf
syntax = "proto3";
package my_pkg.srv.v1;
message AddTwoIntsRequest { int64 a = 1; int64 b = 2; }
message AddTwoIntsResponse { int64 sum = 1; }
```

```bash
protoc --python_out=. --pyi_out=. my_pkg/srv/v1/add_two_ints.proto
```

| Language | Pattern |
|----------|---------|
| Python | Duck-typed methods + `.mapper(MyFoo())` |
| Rust | `impl TypedServiceMapper` / `TypedActionMapper` |
| C++ | `TypedServiceMapper<Derived, RosSrv>` CRTP + `shared_ptr` |

### Python service shape

Implement: `type_name`, `ros_srv_type`, `ros_req_to_bus` / `bus_req_to_ros`, `ros_resp_to_bus` / `bus_resp_to_ros`.
Import generated stubs (`from my_pkg.srv.v1 import add_two_ints_pb2 as pb`) plus the ROS type.

Action: proto Goal / Feedback / Result, then `ros_action_type` + six-way converts — see
`bindings/python/robot_bus/ros2_bridge/mappers/fibonacci.py`.

### Rust service shape

```rust
impl TypedServiceMapper for AddTwoIntsServiceMapper {
    type Ros = my_pkg::srv::AddTwoInts;
    fn type_name(&self) -> &str { "my_pkg/srv/AddTwoInts" }
    fn ros_req_to_bus(&self, req: &Self::Ros::Request) -> robot_bus::Result<Vec<u8>> { /* encode proto */ }
    fn bus_req_to_ros(&self, payload: &[u8]) -> robot_bus::Result<Self::Ros::Request> { /* decode */ }
    fn ros_resp_to_bus(&self, resp: &Self::Ros::Response) -> robot_bus::Result<Vec<u8>> { /* … */ }
    fn bus_resp_to_ros(&self, payload: &[u8]) -> robot_bus::Result<Self::Ros::Response> { /* … */ }
}
```

C++: same four converts on CRTP base; Topic/Action: `TypedTopicMapper` / `TypedActionMapper` in `ros2_bridge_typed.hpp`.

## Runtime model

One process holds:

1. ROS node (rclrs / rclpy / rclcpp)
2. robot-bus `Node`

`spin` / `spin_once` must drive **both** sides (drain ROS↔bus queues + bus).

## Verification

1. `source` ROS; broker up; `ros2_available()` true where applicable.
2. `ros2 topic echo` / `ros2 service call` / `ros2 action send_goal` on ROS names.
3. Bus console `http://127.0.0.1:15570` shows matching bus names with traffic.
4. Wrong direction → silence on one side; flip `Direction` deliberately.

## Common failures

1. ROS not sourced
2. YAML / type-string mounting attempted
3. C++ without `robot_bus_ros2_bridge` → `ros2_available() == false`, `build()` throws
4. Python without `rclpy` on `PYTHONPATH`
5. Missing ROS typesupport for a registered Rust topic mapper

## Deliverables when implementing for the user

1. Bridge program in the chosen language
2. List of routes with direction + mapper (built-in or custom)
3. Proto ↔ ROS field mapping for any custom interfaces
4. Run steps: source ROS + start broker + run bridge + smoke commands

Do not expand full mapper listings here — open [ros2-bridge.md](../zh/ros2-bridge.md) and language API docs.
