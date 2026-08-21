---
name: robot-bus-to-ros2
description: >-
  Convert a robot-bus project/node into a ROS 2 package (rclcpp / rclpy / rclrs),
  or expose bus nodes to a ROS 2 graph via ros2_bridge. Use when the user asks to
  migrate robot-bus to ROS 2, generate ament packages, port protobuf messages to
  .msg/.srv/.action, replace robot_bus_broker with DDS, or graduate a prototype
  onto Humble/Jazzy.
---

# robot-bus → ROS 2

Port robot-bus code onto ROS 2, or keep the bus process and join the ROS graph
via the bridge. Prefer project docs over guessed APIs:
[api-compare.md](../zh/api-compare.md), [ros2-bridge.md](../zh/ros2-bridge.md),
[rust-api.md](../zh/rust-api.md), [python-api.md](../zh/python-api.md),
[cpp-api.md](../zh/cpp-api.md).

Official ROS targets for bridge/interop in this repo: **Humble**, **Jazzy**.

## Decide path first

| Goal | Path |
|------|------|
| Production on Ubuntu with full ROS tooling / nav2 / MoveIt / RViz | **Full migrate** to rclcpp / rclpy / rclrs |
| Keep light bus clients (Android, browser, Windows) talking to ROS | **Bridge** (`BusToRos2` / `Ros2ToBus` as needed) |
| Prototype was on bus; only some nodes need ROS | Hybrid: migrate those packages; bridge shared interfaces |

**Do not** add YAML-configured bridges or string-only type mounts — unsupported.

## Workflow checklist

```
Progress:
- [ ] 1. Inventory bus app (lang, Node APIs, topics/services/actions, protos, params)
- [ ] 2. Choose path: full migrate | bridge | hybrid
- [ ] 3. Map bus protobuf → ROS .msg/.srv/.action (or reuse std ROS types)
- [ ] 4. Create ament package(s) + package.xml / CMakeLists or setup.cfg
- [ ] 5. Rewrite Node / pub-sub / service / action / timer / params for rcl*
- [ ] 6. Replace broker dependency with ROS graph (DDS); drop embedded RobotBusBroker
- [ ] 7. Add launch + remaps; colcon build; ros2 topic/service/action smoke test
```

## 1. Inventory

From the robot-bus project collect:

- Language SDK: Rust / Python / C++ / Java / TypeScript / Android
- How nodes connect: `Node::new` (tcp+discover), `ipc`, `inproc`, `ws` / `ws_at`
- Typed vs raw publishers (`create_publisher::<T>` vs `create_*_raw`)
- Topic / service / action **names** and protobuf type full names (`sensor_msgs.msg.v1.Imu`)
- Custom `.proto` under the app or only built-in bus types
- Local parameters / YAML (`ros__parameters`) — note lack of remote param server on bus
- QoS: only KeepLast depth on topics; reliability is best-effort — when moving to ROS, pick explicit QoS (often `SensorDataQoS` / reliable as appropriate)
- Any existing `ros2_bridge` usage (already half-migrated)

Target ROS client library usually matches the bus language:

| Bus SDK | Typical ROS port |
|---------|------------------|
| Rust | `rclrs` |
| Python | `rclpy` |
| C++ | `rclcpp` |
| Java / TS / Android | Keep on bus + **bridge**, or rewrite in rclcpp/rclpy |

## 2. Runtime & build mapping

| robot-bus | ROS 2 |
|-----------|-------|
| `robot_bus_broker` | DDS + ROS daemon / `ros2` |
| Discover `http://127.0.0.1:15570` | ROS graph discovery |
| `robot-bus` crate / pip / native SDK | `rclcpp` / `rclpy` / `rclrs` + interface packages |
| Process + broker | `ros2 run` / launch |
| Protobuf types in SDK | `.msg` / `.srv` / `.action` + typesupport |
| Web console / `rbus` | `ros2 topic|service|action|param` + RViz |

Create a normal ament package:

```bash
source /opt/ros/humble/setup.bash   # or jazzy
cd src && ros2 pkg create --build-type ament_cmake my_pkg --dependencies rclcpp std_msgs
# or: --build-type ament_python …
```

Remove bus-only pieces: `RobotBusBroker`, `NodeOptions::tcp/discover`, gRPC/WS Node modes, federation `--peer`.

## 3. API rewrite map

Reverse of [api-compare.md](../zh/api-compare.md).

| Concern | robot-bus | ROS 2 |
|---------|-----------|-------|
| Node | `Node::new("name")` | `Node::new(&context, "name")` / `Node("name")` + ROS context/init |
| Spin | `node.spin()` | `rclcpp::spin` / `rclpy.spin` / `rclrs::spin` |
| Publisher | `create_publisher[_with_qos]` | `create_publisher<T>(topic, qos)` |
| Subscription | cb `(topic, msg)` + optional group | cb `msg` (+ qos); groups via executor APIs |
| Service client | `call(req, Some(timeout))` | `call` / async patterns per client lib |
| Action | `send_goal` → GoalHandle → `result` / `cancel` | Same conceptual split; use rcl* action APIs |
| Timer | `create_timer` | `create_wall_timer` / equivalent |
| QoS | KeepLast depth only, best-effort | Full DDS profiles — **choose deliberately** |
| Params | local declare/get/set + YAML | declare/get/set + remote/CLI/launch overrides |

### Rust (rclrs) sketch

```rust
use rclrs::{Context, Node, QOS_PROFILE_DEFAULT};
use std_msgs::msg::String as StringMsg;

fn main() -> Result<(), rclrs::RclrsError> {
    let context = Context::new(std::env::args())?;
    let node = Node::new(&context, "talker")?;
    let publisher = node.create_publisher::<StringMsg>("chatter", QOS_PROFILE_DEFAULT)?;
    let mut msg = StringMsg { data: "hello".into() };
    publisher.publish(&msg)?;
    rclrs::spin(node)
}
```

### Python (rclpy) sketch

```python
import rclpy
from rclpy.node import Node
from std_msgs.msg import String

class Talker(Node):
    def __init__(self):
        super().__init__("talker")
        self.pub = self.create_publisher(String, "chatter", 10)
        self.create_subscription(String, "chatter", self._on_msg, 10)

    def _on_msg(self, msg: String):
        self.get_logger().info(msg.data)

rclpy.init()
node = Talker()
rclpy.spin(node)
```

Preserve absolute names (`/robot1/imu`) unless the ROS package intentionally uses namespaces + remaps.

## 4. Message types

1. If the bus type is a mirror of a standard ROS interface (`sensor_msgs/msg/Imu`, `geometry_msgs/msg/Twist`, `std_srvs/srv/SetBool`, …), **depend on the ROS interface package** and map fields 1:1.
2. Custom bus protobuf → new `my_pkg/msg/Foo.msg` (or srv/action). Keep field names/types aligned to ease future bridge mappers.
3. Bus full name `pkg.msg.v1.Type` → ROS `pkg/msg/Type` (drop `.v1` wire package segment unless you intentionally version ROS msgs).
4. Raw bus topics: define a real ROS message (or `std_msgs/ByteMultiArray` only as last resort).

When both worlds must keep talking, implement a **typed mapper** instead of hoping CDR == protobuf.

## 5. Bridge path (bus stays, ROS joins)

Use when Android/TS/Java clients remain on bus while Ubuntu nodes are ROS:

```text
Ros2Bridge.new(name)
  .bus_tcp(...) | .bus_ipc() | .bus_discover(...)
  .route(ros, bus).mapper(...).direction(BusToRos2|Ros2ToBus).add()
  .service(...).mapper(...).add()
  .action(...).mapper(...).add()
  .build()
  .spin()
```

From [ros2-bridge.md](../zh/ros2-bridge.md):

- Per-route direction only; **no `both`**
- Concrete mapper objects required
- Built-in mappers for String, Image, Trigger, SetBool, Fibonacci; extend with typed converters for custom interfaces
- Run with ROS sourced + broker up; language: Rust `features = ["ros2"]`, Python `rclpy`, C++ `robot_bus_ros2_*` / `ROBOT_BUS_HAS_ROS2`

Pick direction from data ownership:

- Bus publisher → ROS subscribers: `BusToRos2`
- ROS publisher → bus subscribers: `Ros2ToBus`

## 6. What usually needs redesign

- Embedded `RobotBusBroker` / inproc shared `Context` → remove; use ROS executor only
- gRPC / WebSocket `Node` clients → not native ROS; keep on bus behind bridge or rewrite
- Federation peers → ROS domain_id / network design
- Best-effort-only assumptions → set ROS QoS explicitly (especially for command topics that need reliable)
- Local-only parameters → add declare + launch/YAML/`ros2 param` as needed
- Console-centric debugging → teach `ros2 topic echo` / `ros2 interface show`

## 7. Verification

```bash
source /opt/ros/$ROS_DISTRO/setup.bash
colcon build --packages-select <pkg>
source install/setup.bash
ros2 run <pkg> <node>
ros2 topic list
ros2 topic echo /your/topic
ros2 service call …   # if applicable
ros2 action send_goal …  # if applicable
```

If bridging: confirm both `ros2 topic list` and bus console (`http://127.0.0.1:15570`) see the mapped names.

## 8. Deliverables

1. ament package(s) with correct `package.xml` dependencies
2. Ported node sources using rcl* APIs
3. Interface packages for any custom messages (`.msg`/`.srv`/`.action`)
4. Launch file(s) with remaps/params replacing ad-hoc process docs
5. Notes on clients left on bus and any `ros2_bridge` routes required

For signatures and examples, open the linked docs rather than duplicating long samples here.
