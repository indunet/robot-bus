[English](../en/ros2-bridge.md) | 中文

# ROS 2 Bridge（`ros2_bridge`）

进程内把 **ROS 2** 与 **robot-bus** 桥在一起：Topic / Service / Action。

## 架构：三语言各自原生

| 语言 | ROS 客户端 | 入口 | 说明 |
|------|------------|------|------|
| **Rust** | `rclrs` | `robot_bus::ros2_bridge`（Cargo feature **`ros2`**） | Topic 可用 `DynamicMessage`；service/action 走 typed `attach` |
| **Python** | **`rclpy`** | `robot_bus.ros2_bridge` | 纯 Python，**不经** Rust FFI / `rclrs` |
| **C++** | **`rclcpp`** | `<robot_bus/ros2_bridge.hpp>` + `robot_bus_ros2_bridge` | 原生 C++，**不经** Rust FFI / `rclrs` |

```text
Python:  rclpy  ──mapper──► robot_bus.Node
C++:     rclcpp ──mapper──► robot_bus::Node
Rust:    rclrs  ──mapper──► robot_bus::Node
```

**为什么分语言：** ROS service/action 只能用编译期具体类型 create（`create_service<T>` 等）。`rclrs` 没有 topic 那种按字符串建动态 service 的 API。若 C++/Python 只把类型名交给 Rust，`T` 对不上。因此各语言在本侧用具体类型建 ROS 实体，再用本语言 bus `Node` 转发。

| 支持 | 不支持 |
|------|--------|
| Topic / Service / Action | YAML 配置桥 |
| 代码里 `.mapper(具体对象)` | 用类型名字符串 lookup 挂路由 |
| 用户自定义 mapper（本语言具体类型） | 跨语言「只传字符串」的万能桥 |

官方发行版：**Humble**、**Jazzy**。

---

## 前置条件

```bash
source /opt/ros/humble/setup.bash   # 或 jazzy
cargo run --bin robot_bus_broker    # 或已安装的 robot-bus-broker
```

| 语言 | 依赖 |
|------|------|
| 通用 | 可达的 broker（tcp / ipc / discover） |
| Rust | `robot-bus = { features = ["ros2"] }` |
| Python | `robot_bus` + 系统 **`rclpy`**（`just python-dev` / `python-dev-ros2`） |
| C++ | `robot-bus-ros2-humble` 或 `…-jazzy`，或 `just cpp-dev-ros2`（`-DROBOT_BUS_ROS2=ON`，链 **rclcpp**） |

`ros2_available()`：

- **Python**：能否 `import rclpy`
- **C++**：是否以 `ROBOT_BUS_HAS_ROS2` 链接了 `robot_bus_ros2_bridge`
- **Rust FFI / 默认 C ABI**：恒为 false（桥不在 FFI 里）

---

## 统一契约（仅代码）

方向（`Direction`）：`Ros2ToBus`（默认）或 `BusToRos2`，**禁止** `both`。

```text
Ros2Bridge.new / New / new(name)
  .bus_tcp(...) | .bus_ipc() | .bus_discover(...)
  .route(ros, bus).mapper(...).direction(...).add()
  .service(ros, bus).mapper(...).timeout(...).direction(...).add()
  .action(ros, bus).mapper(...).timeout(...).direction(...).add()
  .build()
  .spin() | .spin_once(...)
```

- 默认超时：service **5s**，action goal **30s**
- **无** `from_yaml`；**无** `add_route(..., "pkg/msg/Type", ...)`

### 一期内置 mapper（对象，不是字符串）

| 种类 | Mapper | ROS 类型 |
|------|--------|----------|
| Topic | `StdMsgsStringMapper` | `std_msgs/msg/String` |
| Topic | `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| Service | `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| Service | `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |
| Action | `FibonacciActionMapper` | `example_interfaces/action/Fibonacci` |

Rust 另有完整 topic mapper 注册表（`src/ros2_bridge/mappers/`），挂路由仍须 `.mapper(具体类型)`；`lookup_topic_mapper` / `registered_topic_types` 仅自省，不是挂路由入口。

---

## 用户自定义 service / action：可以

**可以。** 在本语言拿到具体 ROS 类型后，只写 **字段 ↔ bus protobuf 转换**；库负责 `create_service` / client 接线。

| | 行不行 |
|--|--------|
| Python：duck-typed convert 方法 + `.mapper(MyFoo())` | **行** |
| Rust：`impl TypedServiceMapper` / `TypedActionMapper` | **行** |
| C++：`TypedServiceMapper<Derived, RosSrv>` CRTP + `.mapper(shared_ptr)` | **行**（需 `ROBOT_BUS_HAS_ROS2`） |
| 只写 YAML / 类型名字符串 | **不行** |

高级：仍可直接 override `ServiceMapper::attach` / `ActionMapper::attach`（特殊 QoS 等）。

下面用「自有 `my_pkg/srv/AddTwoInts` + 对应 protobuf」说明（字段名示意）。

### Python：自定义 Service mapper

桥调用：`ros_srv_type()`、`ros_req_to_bus` / `bus_req_to_ros`、`ros_resp_to_bus` / `bus_resp_to_ros`。

```python
# from my_pkg.srv import AddTwoInts
# from my_robot.pb.add_two_ints_pb2 import AddTwoIntsRequest, AddTwoIntsResponse

class AddTwoIntsServiceMapper:
    def type_name(self) -> str:
        return "my_pkg/srv/AddTwoInts"

    def ros_srv_type(self):
        return AddTwoInts

    def ros_req_to_bus(self, req) -> bytes:
        return AddTwoIntsRequest(a=int(req.a), b=int(req.b)).SerializeToString()

    def bus_req_to_ros(self, payload: bytes):
        bus = AddTwoIntsRequest()
        bus.ParseFromString(payload)
        out = AddTwoInts.Request()
        out.a = int(bus.a)
        out.b = int(bus.b)
        return out

    def ros_resp_to_bus(self, resp) -> bytes:
        return AddTwoIntsResponse(sum=int(resp.sum)).SerializeToString()

    def bus_resp_to_ros(self, payload: bytes):
        bus = AddTwoIntsResponse()
        bus.ParseFromString(payload)
        out = AddTwoInts.Response()
        out.sum = int(bus.sum)
        return out

bridge = (
    Ros2Bridge.new("bridge")
    .bus_tcp("localhost")
    .service("/add_two_ints", "/add_two_ints")
    .mapper(AddTwoIntsServiceMapper())
    .direction(Direction.Ros2ToBus)
    .timeout(5.0)
    .add()
    .build()
)
```

Action：`ros_action_type()` + `ros_goal_to_bus` / `bus_goal_to_ros` / feedback / result（见 [`mappers/fibonacci.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/fibonacci.py)）。

### Rust：自定义 Service（`TypedServiceMapper`）

```rust
use prost::Message as ProstMessage;
use robot_bus::ros2_bridge::TypedServiceMapper;
// use my_pkg::srv::AddTwoInts;
// use my_robot::pb::{AddTwoIntsRequest, AddTwoIntsResponse};

#[derive(Clone)]
struct AddTwoIntsServiceMapper;

impl TypedServiceMapper for AddTwoIntsServiceMapper {
    type Ros = my_pkg::srv::AddTwoInts;

    fn type_name(&self) -> &str {
        "my_pkg/srv/AddTwoInts"
    }

    fn ros_req_to_bus(&self, req: &Self::Ros::Request) -> robot_bus::Result<Vec<u8>> {
        Ok(AddTwoIntsRequest { a: req.a, b: req.b }.encode_to_vec())
    }

    fn bus_req_to_ros(&self, payload: &[u8]) -> robot_bus::Result<Self::Ros::Request> {
        let bus = AddTwoIntsRequest::decode(payload)?;
        Ok(Self::Ros::Request { a: bus.a, b: bus.b })
    }

    fn ros_resp_to_bus(&self, resp: &Self::Ros::Response) -> robot_bus::Result<Vec<u8>> {
        Ok(AddTwoIntsResponse { sum: resp.sum }.encode_to_vec())
    }

    fn bus_resp_to_ros(&self, payload: &[u8]) -> robot_bus::Result<Self::Ros::Response> {
        let bus = AddTwoIntsResponse::decode(payload)?;
        Ok(Self::Ros::Response { sum: bus.sum })
    }
}

// .service("/add", "/add").mapper(AddTwoIntsServiceMapper).add()?
```

Action：`impl TypedActionMapper`（`type Ros = …` + goal/feedback/result 六向转换）。库内 `wire_typed_*` 负责接线。

### C++：自定义 Service（`TypedServiceMapper` CRTP）

内置仍用 ZST：`.mapper(TriggerServiceMapper{})`。自定义继承 CRTP，只写转换；库自动 `attach` / `retain`。

```cpp
#include <robot_bus/ros2_bridge.hpp>
// #include <my_pkg/srv/add_two_ints.hpp>
// #include "add_two_ints.pb.h"

struct AddTwoIntsServiceMapper
    : robot_bus::TypedServiceMapper<AddTwoIntsServiceMapper, my_pkg::srv::AddTwoInts> {
  const char *type_name() const override { return "my_pkg/srv/AddTwoInts"; }

  std::vector<uint8_t> ros_req_to_bus(const Request &req) const { /* … */ }
  Request bus_req_to_ros(robot_bus::BytesView body) const { /* … */ }
  std::vector<uint8_t> ros_resp_to_bus(const Response &resp) const { /* … */ }
  Response bus_resp_to_ros(robot_bus::BytesView body) const { /* … */ }
};

// .service("/add", "/add")
//     .mapper(std::make_shared<AddTwoIntsServiceMapper>())
//     .direction(robot_bus::Direction::Ros2ToBus)
//     .add()
```

Topic / Action：`TypedTopicMapper` / `TypedActionMapper`（见 [`ros2_bridge_typed.hpp`](../../bindings/cpp/include/robot_bus/ros2_bridge_typed.hpp)）。

---

## Rust（`rclrs`）

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

- 自定义 topic：`impl TopicMapper` + `mapper_support`（`DynamicMessage`）
- 自定义 service/action：`TypedServiceMapper` / `TypedActionMapper`（见上文「用户自定义」）
- 模块：`typed_service`（`wire_typed_*` / `attach_*`）、`dynamic_rpc::spike_summary()`

---

## Python（`rclpy`）

实现目录：[`bindings/python/robot_bus/ros2_bridge/`](../../bindings/python/robot_bus/ros2_bridge/)（纯 Python）。

```bash
source /opt/ros/humble/setup.bash
just python-dev-ros2   # 或 just python-dev；需本机有 rclpy
```

```python
import robot_bus
from robot_bus.ros2_bridge import (
    Direction,
    Ros2Bridge,
    StdMsgsStringMapper,
    TriggerServiceMapper,
)

assert robot_bus.ros2_available()  # import rclpy 成功

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

- ROS 侧：`rclpy` 节点 + executor（后台线程 spin）
- Bus 侧：`robot_bus.Node`（raw / typed protobuf）
- 自定义 mapper：见上文「用户自定义」；内置参考 [`mappers/trigger.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/trigger.py)
- Mapper 按需 lazy import（依赖对应 ROS 消息包与 protobuf）

---

## C++（`rclcpp`）

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

- 链接 **`robot_bus_ros2_bridge`**（`ROBOT_BUS_HAS_ROS2`）；无此宏时 `build()` 抛错
- 本机构建：`just cpp-dev-ros2`（需先 `just gen-cpp` + source ROS）
- 包：`robot-bus-ros2-humble` / `robot-bus-ros2-jazzy`（**不** vendor `rcl`）
- 内置 ZST + `.mapper(std::shared_ptr<…Mapper>)` 自定义；见上文「用户自定义」

---

## 运行时

同进程同时持有：

1. ROS 节点（rclrs / rclpy / rclcpp）
2. robot-bus `Node`

主循环需推进两侧（`spin` / `spin_once`）；各语言实现细节不同，语义一致：排空 ROS↔bus 队列并驱动 bus。

---

## 常见问题

1. **未 source ROS** — 三端都会失败。
2. **YAML 配桥** — 不支持；代码里挂 mapper。
3. **只传类型名字符串** — 不支持挂路由；传具体 mapper 对象。
4. **想跨语言万能动态 srv** — 不做；在目标语言写自定义 mapper。
5. **C++ `ros2_available() == false`** — 未链 `robot_bus_ros2_bridge` / 装的是无桥包。
6. **Python `ros2_available() == False`** — 未安装或未 source 到 `rclpy`。
7. **Rust topic 登记了但跑不起来** — 缺对应 ROS typesupport（如 `foxglove_msgs`）。

---

## 相关

- C++ 包与本地构建：[cpp-api.md](cpp-api.md)
- Python SDK：[python-api.md](python-api.md)
- API 对比：[api-compare.md](api-compare.md)
