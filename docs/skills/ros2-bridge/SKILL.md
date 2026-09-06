---
name: ros2-bridge
description: >-
  Create and run robot-bus Ros2Bridge (topic / service / action) between ROS 2
  Humble or Jazzy and robot_bus_broker. Use when the user asks to bridge ROS and
  bus, mount from_ros/to_bus (TopicQos on every endpoint; bus must be
  best_effort), custom TypedServiceMapper / TypedActionMapper / TypedTopicMapper,
  or connect rclcpp/rclpy/rclrs to bus without a full package migration.
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
| Every endpoint: name + `TopicQos` preset (`default` / `sensor_data` / `latched` / `bus`) or `keep_last(n).reliable()` / `.best_effort()` | Native `rclpy`/`rclcpp` QoS as the bridge second arg |
| ROS service/action: same `TopicQos` on `from_ros` / `to_ros` | Silent default ROS service QoS |
| Bus `TopicQos` on topics, services, and actions; must be `.best_effort()` | Silent drop of `.reliable()` on bus |
| Mount with `.mapper(concrete object)` | Type-name string lookup to mount routes |
| Service/action: `.service().from_ros(name, TopicQos).to_bus(name, TopicQos)` | `.service(ros, bus)` or `.direction()` |
| Native bridge per language (rclrs / rclpy / rclcpp) | Cross-language “string-only” universal bridge |

Official ROS targets: **Humble**, **Jazzy**.

## Decide direction

Direction is the chain (`from_ros → to_bus` vs `from_bus → to_ros`) for topics, services, and actions:

| Data ownership | Chain |
|----------------|-------|
| ROS publishes / serves → bus clients consume | `from_ros(...).to_bus(...)` |
| Bus publishes / serves → ROS subscribers / clients | `from_bus(...).to_ros(...)` |

Start services with `.service()` and actions with `.action()`. Pass `TopicQos` on every name (`from_ros` / `to_ros` / `from_bus` / `to_bus`). Same logical name on both sides is fine (`/chatter`, `/chatter`); names need not match. Prefer presets: `TopicQos.default()` (C++ `ros_default()`) for ordinary ROS topics/services/actions, `sensor_data()` for camera/lidar, `latched()` for `/tf_static`, `bus()` on every bus endpoint. Custom depth still uses `keep_last(n).reliable()` / `.best_effort()`. ROS accepts reliable or best-effort; bus endpoints must be `.best_effort()` (`default()` / `latched()` are rejected on bus).

## Prerequisites

```bash
source /opt/ros/humble/setup.bash   # or jazzy
# Prefer RobotBusBroker.start() in application code; CLI for a standalone broker
python -m robot_bus.broker                    # or cargo run --bin robot_bus_broker
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
- [ ] 5. Ros2Bridge.new → bus_* → from_ros/to_bus (or from_bus/to_ros); pass TopicQos on both ends for topic, service, and action; service/action: `.service().from_ros(name, TopicQos).to_bus(name, TopicQos).mapper().timeout().add()`
- [ ] 6. bridge.spin(); verify ros2 topic echo + bus console
```

## Unified builder contract

```text
.from_ros(ros, TopicQos).to_bus(bus, TopicQos).mapper(...).lazy()?.add()
.from_bus(bus, TopicQos).to_ros(ros, TopicQos).mapper(...).add()
.service().from_ros(ros, TopicQos).to_bus(bus, TopicQos).mapper(...).timeout(...).add()
.action().from_ros(ros, TopicQos).to_bus(bus, TopicQos).mapper(...).timeout(...).add()
```

Usual `TopicQos`: `default()` / `sensor_data()` / `latched()` / `bus()` (C++ `ros_default()` instead of `default()`). Custom depth: `keep_last(n)` then **must** `.reliable()` or `.best_effort()`. Bus endpoints (topic, service, action) must be `.best_effort()`. Direction is the chain. **No `both`**. Action applies the ROS profile to goal/result/cancel + feedback; status stays ROS default. Bus RPC depth → DEALER HWM. Defaults: service timeout **5s**, action goal **30s**. Topic routes are **eager** at `build()`; `.lazy()` is opt-in on `from_ros → to_bus` topics only (camera/lidar). No-console brokers fall back to eager. `from_bus → to_ros` and service/action have no `.lazy()`.

### Built-in mappers (objects, not strings)

**Topics** share one catalog across Rust / Python / C++ (~125 types from Humble/Jazzy **distro-common** packages; allowlist `CORE_BRIDGE_PACKAGES` in `scripts/generate_topic_mappers.py`). After proto changes, re-run `just gen-topic-mappers`. Mount with `.mapper(GeometryMsgsPoseStampedMapper())` (Python) / `.mapper(GeometryMsgsPoseStampedMapper{})` (C++; same attach path as custom mappers; C++ `TopicBuiltin` stays String/Image only).

**Not builtins:** `nav2_msgs` / `control_msgs` / `foxglove_msgs` / `apriltag_msgs`. Keep using bus protos if present; write TypedTopicMapper for ROS bridging. Later `apt install` of those packages does not enable bridge mappers.

**Service / action** remain hand-written builtins (no generated srv/action catalog):

| Kind | Mapper | ROS type |
|------|--------|----------|
| Topic (example) | `StdMsgsStringMapper` | `std_msgs/msg/String` |
| Topic (example) | `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| Topic (example) | `GeometryMsgsPoseStampedMapper` | `geometry_msgs/msg/PoseStamped` |
| Service | `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| Service | `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |
| Action | `FibonacciActionMapper` | `example_interfaces/action/Fibonacci` |

Rust: `lookup_topic_mapper` / `registered_topic_types` for the core set. **Mounting still requires** `.mapper(concrete object)`, not a type-name string.

## Minimal examples

### Python

```python
import robot_bus
from robot_bus.ros2_bridge import (
    Ros2Bridge, StdMsgsStringMapper, TopicQos, TriggerServiceMapper,
)

assert robot_bus.ros2_available()

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
bridge.spin()
```

### Rust

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

### C++

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
| Rust | `impl TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` |
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
3. Bus console `http://127.0.0.1:15560` **BRIDGE** tab shows routes, `drop_stats`, and idle; Topics may mark `bridged`.
4. Wrong direction or ROS QoS mismatch → idle WARN after ~15s (`possible wrong direction or ROS QoS mismatch`); flip the chain or QoS.

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
