---
name: ros2-to-robot-bus
description: >-
  Convert a ROS 2 package/project (rclcpp / rclpy / rclrs) into a robot-bus
  project, or wire coexistence via ros2_bridge. Use when the user asks to migrate
  from ROS 2 to robot-bus, port ament packages, replace DDS with robot_bus_broker,
  rewrite nodes for robot-bus APIs, or convert .msg/.srv/.action to bus protobuf.
---

# ROS 2 → robot-bus

Migrate ROS 2 code to robot-bus, or keep ROS 2 and interconnect via the bridge.
Read project docs under `docs/zh/` (or `docs/en/`) before inventing APIs:
[api-compare.md](../zh/api-compare.md), [ros2-bridge.md](../zh/ros2-bridge.md),
[rust-api.md](../zh/rust-api.md), [python-api.md](../zh/python-api.md),
[cpp-api.md](../zh/cpp-api.md).

## Decide path first

| Goal | Path |
|------|------|
| Drop ROS distro / DDS; run on broker only | **Full migrate** |
| Keep ROS graph; add Android / browser / Windows / light nodes | **Bridge** (`ros2_bridge`) |
| Gradual cutover | Bridge first for shared topics/services/actions, then migrate nodes one by one |

**Do not** invent YAML bridge configs or type-name-string route mounting — not supported.

## Workflow checklist

```
Progress:
- [ ] 1. Inventory source package (lang, nodes, topics/services/actions, msgs, params, launch)
- [ ] 2. Choose path: full migrate | bridge | hybrid
- [ ] 3. Map message types (.msg/.srv/.action → bus protobuf or custom proto)
- [ ] 4. Replace build/runtime (ament → Cargo / pip / CMake+robot-bus)
- [ ] 5. Rewrite Node / pub-sub / service / action / timer / params
- [ ] 6. Ensure broker (external `robot_bus_broker` or in-process)
- [ ] 7. Build & smoke-test with console / rbus / language tests
```

## 1. Inventory

From the ROS 2 package collect:

- Language: `rclcpp` | `rclpy` | `rclrs`
- Entry points: `package.xml`, `CMakeLists.txt` / `setup.py` / `Cargo.toml`, launch files
- Graph: topic names + types, services, actions, remaps, namespaces
- Custom interfaces in `msg/` `srv/` `action/`
- Parameters (declare / YAML / CLI `-p`) and timers / callback groups
- Dependencies that are ROS-only (tf2, nav2, MoveIt, plugins) — flag as **stay on ROS** or need redesign

Prefer keeping **topic / service / action names** stable so a bridge or gradual cutover stays simple.

## 2. Runtime & build mapping

| ROS 2 | robot-bus |
|-------|-----------|
| DDS + `ros2` daemon | `robot_bus_broker` (tcp/ipc) or embedded `RobotBusBroker` |
| `source /opt/ros/<distro>/setup.bash` | Not required for pure bus apps |
| ament workspace / colcon | Language package manager + bus SDK |
| `ros2 run` / launch | Process + broker; optional process supervisor |
| Message codegen from `.msg` | Crate/SDK protobuf (`sensor_msgs.msg.v1.*` etc.) or project `.proto` |

Broker quick start:

```bash
# installed
robot-bus-broker
# or from this repo
cargo run --bin robot_bus_broker
```

Default API / console: `http://127.0.0.1:15570`. `Node` defaults to tcp + discover against that API.

**Inproc** (same process as embedded broker): share one `Context` — see rust-api.md. Cross-process tcp/ipc does not need shared context.

## 3. API rewrite map

Canonical side-by-side examples: [api-compare.md](../zh/api-compare.md).

| Concern | ROS 2 (typical) | robot-bus |
|---------|-----------------|-----------|
| Node | `Node::new(&ctx, "name")` / `Node("name")` | `Node::new("name")` / `Node("name")` |
| Spin | `rclcpp::spin` / `rclpy.spin` / `rclrs::spin` | `node.spin()` (auto SingleThreadedExecutor) |
| Publisher | `create_publisher<T>(topic, qos)` | `create_publisher` / `create_publisher_with_qos` |
| Subscription | `(topic, qos, cb)` | `(topic, qos?, cb, callback_group?)` — bus cb often gets `(topic, msg)` |
| Service | `create_service` / `create_client` | same names; client `call` takes timeout |
| Action | `ActionServer` / `ActionClient` | `create_action_server` / `create_action_client` + GoalHandle |
| Timer | `create_wall_timer` | `create_timer(period, cb, group?)` |
| QoS | full DDS QoS | Topic only: `QosProfile::keep_last(depth)` → ZMQ HWM / WS subscribe queue; **reliability fixed best-effort**. WS publish ignores QoS. Service/action: no QoS yet |
| Params | declare/get/set + remote/CLI | Same local shape + YAML load; **no** remote param server / CLI `-p` |
| Callback groups | MutuallyExclusive / Reentrant | `CallbackGroupType::MutuallyExclusive` / `Reentrant` |

### Rust sketch

```rust
use robot_bus::{Node, QosProfile};
use robot_bus::sensor_msgs::msg::v1::Imu;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("talker");
    let pub_ = node.create_publisher_with_qos::<Imu>("/robot1/imu", QosProfile::keep_last(10))?;
    node.create_subscription_with_qos::<Imu, _>(
        "/robot1/imu",
        QosProfile::keep_last(10),
        |_topic, imu| { let _ = imu; },
        None,
    )?;
    pub_.publish(&Imu::default())?;
    node.spin()?;
    Ok(())
}
```

`Cargo.toml`: `robot-bus = "1.3.2"` (match current crate version). Bridge needs `features = ["ros2"]`.

### Python sketch

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu

node = robot_bus.Node("talker")
pub = node.create_publisher("/robot1/imu", Imu)
node.create_subscription("/robot1/imu", Imu, lambda topic, msg: None)
pub.publish(Imu())
node.spin()
```

### C++

Follow [cpp-api.md](../zh/cpp-api.md). ROS bridge package: `robot-bus-ros2-humble` / `robot-bus-ros2-jazzy` or `just cpp-dev-ros2`.

## 4. Message types

1. Prefer **built-in** bus types that mirror common ROS packages: `std_msgs`, `geometry_msgs`, `sensor_msgs`, `nav_msgs`, `std_srvs`, `tf2_msgs`, `visualization_msgs`, … (see `proto/` and SDK namespaces `*.msg.v1` / `*.srv.v1` / `action.v1`).
2. Custom `.msg` / `.srv` / `.action` → add project protobuf with equivalent fields; regenerate language bindings.
3. Naming: bus uses protobuf full names like `sensor_msgs.msg.v1.Imu` (not `sensor_msgs/msg/Imu`).
4. Raw bytes escape hatch: `create_*_raw` when you must carry opaque payloads.

Field mapping tips:

- `std_msgs/Header` → bus header / timestamp fields as defined in the matching proto
- Fixed-length arrays → repeated or bytes per proto definition
- Do not assume DDS CDR wire format on the bus — always encode/decode via SDK typed APIs or protobuf

## 5. Bridge path (keep ROS + talk to bus)

When only interconnect is needed, **do not** rewrite the whole graph. Use `Ros2Bridge` in the language that owns the ROS types:

```text
Ros2Bridge.new(name)
  .bus_tcp(...) | .bus_ipc() | .bus_discover(...)
  .route(ros, bus).mapper(...).direction(Ros2ToBus|BusToRos2).add()
  .service(...).mapper(...).timeout(...).add()
  .action(...).mapper(...).timeout(...).add()
  .build()
  .spin()
```

Rules (from [ros2-bridge.md](../zh/ros2-bridge.md)):

- Direction is one-way per route; **no `both`**
- Mount with **concrete mapper objects**, not type-name strings
- Built-ins: `StdMsgsStringMapper`, `SensorMsgsImageMapper`, `TriggerServiceMapper`, `SetBoolServiceMapper`, `FibonacciActionMapper`
- Custom service/action: implement typed field converters (`TypedServiceMapper` / duck-typed Python / C++ CRTP)
- Prerequisites: `source` ROS Humble or Jazzy; broker reachable; language-specific ROS feature/package

Default: `Direction::Ros2ToBus` when ROS is the publisher/server side feeding bus clients.

## 6. What usually does NOT migrate 1:1

- Launch XML/Python → rewrite as process list / compose / scripts
- `ros2 topic/service/action` CLI → Web console + `rbus` + language smoke tests
- Lifecycle nodes / components / pluginlib → redesign on bus or keep on ROS
- tf2 buffer / transform tree → keep on ROS behind the bridge; robot-bus has no TF library
- Reliable / deadline / liveliness QoS — bus topics are best-effort KeepLast depth only
- Remote parameters and `ros2 param` — local YAML/API only

## 7. Verification

1. Start broker; open `http://127.0.0.1:15570` — topics/services/actions appear when nodes run.
2. Publish/subscribe once end-to-end; call one service; send one action goal if used.
3. If bridging: source ROS, confirm `ros2 topic echo` / bus console both see traffic on mapped names.
4. Match original remaps/namespaces deliberately; prefer absolute names like `/robot1/imu`.

## 8. Deliverables

When migrating a package, produce:

1. New project layout (Cargo/pip/CMake) depending on robot-bus (no ament required for pure bus)
2. Ported node sources with API mapping applied
3. Proto additions for any custom interfaces (if needed)
4. Short run instructions: how to start broker + nodes
5. Explicit list of **unported** ROS features and recommended bridge/hybrid leftovers

For detailed API snippets and more languages, read the docs linked above rather than expanding this skill inline.
