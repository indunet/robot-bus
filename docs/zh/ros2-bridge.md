[English](../en/ros2-bridge.md) | 中文

# ROS2 Bridge（`ros2_bridge`）

进程内把 **ROS2** 与 **robot-bus** 桥在一起。Python / C++ / Rust 各自用本语言 ROS 客户端（`rclpy` / `rclcpp` / `rclrs`），不经 FFI。

| 语言 | ROS客户端 | 入口 |
|------|------------|------|
| **Rust** | `rclrs` | `robot_bus::ros2_bridge`（Cargo feature **`ros2`**） |
| **Python** | `rclpy` | `robot_bus.ros2_bridge` |
| **C++** | `rclcpp` | `<robot_bus/ros2_bridge.hpp>` + `robot_bus_ros2_bridge` |

官方发行版：**Humble**、**Jazzy**。一条桥进程里可以同时挂多条话题 / 服务 / action。方向只能单向：`from_ros → to_bus` 或 `from_bus → to_ros`，没有 `both`。

可运行示例：[`examples/ros2_bridge/`](../../examples/ros2_bridge/)（`builtin` 为内置 mapper，`custom_add_two_ints` 为自定义服务）。

---

## 前置条件

```bash
source /opt/ros/humble/setup.bash   # 或 jazzy
# 应用代码更鼓励进程内 RobotBusBroker.start()；下面 CLI 适合单独起 broker 再跑桥
cargo run --bin robot_bus_broker    # 或 python -m robot_bus.broker
```

| 语言 | 依赖 |
|------|------|
| 通用 | 可达的 broker（tcp / ipc / discover） |
| Rust | `robot-bus = { features = ["ros2"] }` + 下文「Rust overlay」 |
| Python | `robot_bus` + 系统 **`rclpy`**（`just python-dev` / `python-dev-ros2`） |
| C++ | `robot-bus-ros2-humble` 或 `…-jazzy`，或 `just cpp-dev-ros2`（`-DROBOT_BUS_ROS2=ON`） |

`ros2_available()`：Python 看能否 `import rclpy`；C++ 看是否以 `ROBOT_BUS_HAS_ROS2` 链了 `robot_bus_ros2_bridge`；Rust FFI / 默认 C ABI 恒为 false（桥不在 FFI 里）。

两端都写 **名字 + `TopicQos`**。常用命名预设：`default()`（C++ 为 `ros_default()`，因为 `default` 是关键字）、`sensor_data()`、`latched()`、`bus()`。自定义深度仍用 `keep_last(n).reliable()` / `.best_effort()`。ROS 端 reliable / best-effort 都行；**bus 端只能 `.best_effort()`**（没有 DDS reliability，depth 只变成 HWM），因此 `default()` / `latched()` 不能写在 bus 端。`default()` 对应 ROS `qos_profile_default` / `ServicesQoS`，**不是** ROS `SystemDefaultsQoS`。

---

## 话题

ROS 在发、bus 侧要订：`.from_ros → .to_bus`。bus 在发、ROS 侧要订：`.from_bus → .to_ros`。挂上内置 mapper，`build()` 后 `spin()`。

```python
from robot_bus.ros2_bridge import Ros2Bridge, StdMsgsStringMapper, TopicQos

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", TopicQos.default())
    .to_bus("/chatter", TopicQos.bus())
    .mapper(StdMsgsStringMapper())
    .add()
    .build()
)
bridge.spin()
```

```rust
use robot_bus::ros2_bridge::{Ros2Bridge, StdMsgsStringMapper, TopicQos};

let mut bridge = Ros2Bridge::new("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", TopicQos::default())
    .to_bus("/chatter", TopicQos::bus())
    .mapper(StdMsgsStringMapper)
    .add()?
    .build()?;
bridge.spin()?;
```

```cpp
#include <robot_bus/ros2_bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", robot_bus::TopicQos::ros_default())
    .to_bus("/chatter", robot_bus::TopicQos::bus())
    .mapper(robot_bus::StdMsgsStringMapper{})
    .add()
    .build();
bridge.spin();
```

反向（bus → ROS）把链改成 `.from_bus("/chatter", …).to_ros("/chatter", …)`，其余一样。两边名字可以相同，也可以不同。

一条桥里连续挂多条话题：每条各自 `.from_ros` / `.to_bus`（或反过来）→ `.mapper(...)` → `.add()`，再开下一条。第二条话题**不用** `.service()` / `.action()`（那是切种类）。方向可以混。

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

相机 / lidar 对上 ROS `SensorDataQoS` 时用 `TopicQos.sensor_data()`（KeepLast 5, best effort）。bus 端也可以写 `TopicQos.bus()`，或同样用 `sensor_data()` 让 HWM=5。

`/tf_static` 这类 latch 话题用 `TopicQos.latched()`（KeepLast 1, reliable, transient local），否则默认 volatile 订不到已经发过的样本：

```python
.from_ros("/tf_static", TopicQos.latched())
.to_bus("/tf_static", TopicQos.bus())
```

自定义深度仍可手写 `keep_last(n)…`，两端可以混用预设和手写。C++ 若要把 latch 改回 volatile，方法名是 `.durability_volatile()`（`volatile` 是关键字）。bus 端没有 DDS durability，写了也会被忽略。

Topic mapper 三语言同一套目录（Humble/Jazzy **发行版常见自带**接口包，约 125 个类型）：Rust `src/ros2_bridge/mappers/`，Python `from robot_bus.ros2_bridge import GeometryMsgsPoseStampedMapper`，C++ `#include <robot_bus/ros2_bridge.hpp>`（伞头文件 `ros2_bridge_topic_mappers.hpp`）。包集合见 `scripts/generate_topic_mappers.py` 的 `CORE_BRIDGE_PACKAGES`。改 proto 后跑 `just gen-topic-mappers`。挂路由直接 `.mapper(GeometryMsgsPoseStampedMapper())` / `.mapper(GeometryMsgsPoseStampedMapper{})`。

**不作为桥内置**：`nav2_msgs` / `control_msgs` / `foxglove_msgs` / `apriltag_msgs` 等扩展栈。仓库里仍可有对应 **proto**（bus 原生可用），但默认不提供 ROS↔bus mapper；需要时请自写 `TypedTopicMapper`。后装 apt 扩展包**不会**自动点亮桥能力。

| 示例 mapper | ROS 类型 |
|-------------|----------|
| `StdMsgsStringMapper` | `std_msgs/msg/String` |
| `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| `GeometryMsgsPoseStampedMapper` | `geometry_msgs/msg/PoseStamped` |

Rust 另有 `lookup_topic_mapper` / `registered_topic_types`（仅核心集）。

### `.lazy()`（大流量 ROS → bus）

默认 eager：`build()` 立刻建 ROS subscription，ROS 图上能看到这座桥。相机、雷达等大流量才在 `from_ros → to_bus` 上加 `.lazy()`：没人订 bus 时，ROS 图上这座桥不是该 topic 的 subscriber。

```python
.from_ros("/camera/image", TopicQos.sensor_data())
.to_bus("/camera/image", TopicQos.sensor_data())
.mapper(SensorMsgsImageMapper())
.lazy()
.add()
```

Rust / C++ 同样是无参 `.lazy()`。`from_bus → to_ros`、服务、action 都没有 `.lazy()`。`--no-console` 的 broker 没有 demand 信号，lazy 路由会降级成 eager。需求只数 `kind == subscriber`；裸 `Subscriber`（不经 `Node`）以及关掉 topology 的 WebSocket 打不开 lazy。C++ 只 override `attach` 的自定义 mapper 不支持 `.lazy()`，请用 `TypedTopicMapper`。

broker 在 subscriber register/unregister 时立刻往 `/robot_bus/topic_demand` 发 [`TopicDemand`](../../proto/robot_bus_interfaces/msg/v1/console_status.proto)。桥启动时再读 `/robot_bus/topics`，避免订阅者先于桥启动时 lazy 路由一直关着。

### 自定义话题 mapper

先写对齐 ROS `.msg` 的 bus protobuf，`protoc` 生成本语言 stubs，再只写字段转换；库负责 `create_subscription` / `create_publisher`。不必把类型放进 robot-bus 仓库。

**Python**（duck-typed）：

```python
from std_msgs.msg import String as RosString
from robot_bus.std_msgs.msg.v1 import String as BusString

class MyStringMapper:
    def ros_msg_type(self):
        return RosString

    def ros_to_bus(self, msg) -> bytes:
        return BusString(data=msg.data).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        bus = BusString()
        bus.ParseFromString(payload)
        out = RosString()
        out.data = bus.data
        return out
```

**Rust**（`TypedTopicMapper`）：

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

**C++**（`TypedTopicMapper` CRTP）：继承 `robot_bus::TypedTopicMapper<MyMapper, ros_msgs::msg::MyMsg>`，实现 `ros_to_bus` / `bus_to_ros`，见 [`ros2_bridge_typed.hpp`](../../bindings/cpp/include/robot_bus/ros2_bridge_typed.hpp)。

---

## 服务

先 `.service()`，再写两端名字 + QoS。ROS 对上 `services_default` 写 `TopicQos.default()`（C++ `ros_default()`）；bus 写 `TopicQos.bus()`（depth → DEALER HWM）。默认超时 **5s**，要用 `.timeout(...)` 改。

ROS 提供服务、bus 客户端去调：`.from_ros → .to_bus`。bus 提供服务、ROS 客户端去调：`.from_bus → .to_ros`。

```python
.service()
.from_ros("/reset", TopicQos.default())
.to_bus("/reset", TopicQos.bus())
.mapper(TriggerServiceMapper())
.timeout(5.0)
.add()
```

```rust
.service()
    .from_ros("/reset", TopicQos::default())
    .to_bus("/reset", TopicQos::bus())
    .mapper(TriggerServiceMapper)
    .timeout(std::time::Duration::from_secs(5))
    .add()?
```

```cpp
.service()
    .from_ros("/reset", robot_bus::TopicQos::ros_default())
    .to_bus("/reset", robot_bus::TopicQos::bus())
    .mapper(robot_bus::TriggerServiceMapper{})
    .add()
```

内置服务 mapper（手写，不生成全量 srv 目录）：

| Mapper | ROS 类型 |
|--------|----------|
| `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |

### 自定义服务 mapper

先写对齐 ROS `.srv` 的 bus protobuf，再实现 request / response 四向转换。库负责接线。

以 ROS `example_interfaces/srv/AddTwoInts` 为例（与工程内自有 `my_pkg/srv/AddTwoInts` 写法相同）。本仓库已提供 [`proto/example_interfaces/srv/v1/add_two_ints.proto`](../../proto/example_interfaces/srv/v1/add_two_ints.proto)。完整可运行文件：[`examples/ros2_bridge/python/custom_add_two_ints.py`](../../examples/ros2_bridge/python/custom_add_two_ints.py)。

```text
# example_interfaces/srv/AddTwoInts.srv
int64 a
int64 b
---
int64 sum
```

```protobuf
syntax = "proto3";
package example_interfaces.srv.v1;

message AddTwoIntsRequest {
  int64 a = 1;
  int64 b = 2;
}

message AddTwoIntsResponse {
  int64 sum = 1;
}
```

工程内自有类型自行 `protoc`：

```bash
# Python
protoc --python_out=. --pyi_out=. my_pkg/srv/v1/add_two_ints.proto

# C++
protoc --cpp_out=. my_pkg/srv/v1/add_two_ints.proto
```

Rust 在 `build.rs` 里：

```rust
prost_build::compile_protos(
    &["proto/my_pkg/srv/v1/add_two_ints.proto"],
    &["proto"],
)?;
```

**Python**：实现 `ros_srv_type()`、`ros_req_to_bus` / `bus_req_to_ros`、`ros_resp_to_bus` / `bus_resp_to_ros`。

```python
from example_interfaces.srv import AddTwoInts
from robot_bus.example_interfaces.srv.v1 import add_two_ints_pb2 as pb
from robot_bus.ros2_bridge import Ros2Bridge, TopicQos

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

**Rust**（`TypedServiceMapper`），可运行：[`examples/ros2_bridge/rust/custom_add_two_ints.rs`](../../examples/ros2_bridge/rust/custom_add_two_ints.rs)。

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

// .service().from_ros("/examples/add_two_ints", TopicQos::default())
//     .to_bus("/examples/add_two_ints", TopicQos::bus())
//     .mapper(AddTwoIntsServiceMapper)
//     .add()?
```

**C++**（`TypedServiceMapper` CRTP），可运行：[`examples/ros2_bridge/cpp/custom_add_two_ints.cpp`](../../examples/ros2_bridge/cpp/custom_add_two_ints.cpp)。内置仍用 ZST：`.mapper(TriggerServiceMapper{})`；自定义继承 CRTP，`.mapper(std::make_shared<AddTwoIntsServiceMapper>())`。

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
```

---

## Action

先 `.action()`，再写两端名字 + QoS。ROS 这份 profile 用在 goal / result / cancel 三个 service 以及 feedback topic；status topic 保持 ROS action-status 默认。默认 goal 超时 **30s**。

ROS 是 action server、bus 客户端去发 goal：`.from_ros → .to_bus`。bus 是 action server、ROS 客户端去发 goal：`.from_bus → .to_ros`。

```python
.action()
.from_ros("/fibonacci", TopicQos.default())
.to_bus("/fibonacci", TopicQos.bus())
.mapper(FibonacciActionMapper())
.add()
```

```rust
.action()
    .from_ros("/fibonacci", TopicQos::default())
    .to_bus("/fibonacci", TopicQos::bus())
    .mapper(FibonacciActionMapper)
    .add()?
```

```cpp
.action()
    .from_ros("/fibonacci", robot_bus::TopicQos::ros_default())
    .to_bus("/fibonacci", robot_bus::TopicQos::bus())
    .mapper(robot_bus::FibonacciActionMapper{})
    .add()
```

内置只有 `FibonacciActionMapper`（`example_interfaces/action/Fibonacci`）。

### 自定义 action mapper

proto 里写 Goal / Feedback / Result 三个 message，再实现六向转换。库负责 action server / client 接线。参考 [`mappers/fibonacci.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/fibonacci.py)。

```protobuf
syntax = "proto3";
package example_interfaces.action.v1;

message FibonacciGoal { int32 order = 1; }
message FibonacciFeedback { repeated int32 sequence = 1; }
message FibonacciResult { repeated int32 sequence = 1; }
```

**Python**：`ros_action_type()` + `ros_goal_to_bus` / `bus_goal_to_ros` / `ros_feedback_to_bus` / `bus_feedback_to_ros` / `ros_result_to_bus` / `bus_result_to_ros`。

```python
from example_interfaces.action import Fibonacci
from robot_bus.example_interfaces.action.v1 import (
    FibonacciFeedback,
    FibonacciGoal,
    FibonacciResult,
)

class MyFibonacciMapper:
    def type_name(self) -> str:
        return "example_interfaces/action/Fibonacci"

    def ros_action_type(self):
        return Fibonacci

    def ros_goal_to_bus(self, goal) -> bytes:
        return FibonacciGoal(order=int(goal.order)).SerializeToString()

    def bus_goal_to_ros(self, payload: bytes):
        bus = FibonacciGoal()
        bus.ParseFromString(payload)
        out = Fibonacci.Goal()
        out.order = bus.order
        return out

    def ros_feedback_to_bus(self, feedback) -> bytes:
        return FibonacciFeedback(sequence=list(feedback.sequence)).SerializeToString()

    def bus_feedback_to_ros(self, payload: bytes):
        bus = FibonacciFeedback()
        bus.ParseFromString(payload)
        out = Fibonacci.Feedback()
        out.sequence = list(bus.sequence)
        return out

    def ros_result_to_bus(self, result) -> bytes:
        return FibonacciResult(sequence=list(result.sequence)).SerializeToString()

    def bus_result_to_ros(self, payload: bytes):
        bus = FibonacciResult()
        bus.ParseFromString(payload)
        out = Fibonacci.Result()
        out.sequence = list(bus.sequence)
        return out
```

**Rust**：`impl TypedActionMapper`（`type Ros = …` + 同样六向转换）。**C++**：继承 `TypedActionMapper<Derived, RosAction>`，见 [`ros2_bridge_typed.hpp`](../../bindings/cpp/include/robot_bus/ros2_bridge_typed.hpp)。特殊 QoS 仍可直接 override `ActionMapper::attach`。

---

## 各语言怎么跑

### Rust（`rclrs`）

`feature = "ros2"` 需要 `AMENT_PREFIX_PATH` 上有核心桥包的 rust IDL（`share/<pkg>/rust/`）。Humble 上 `common_interfaces` 等常见包通常已自带；`source /opt/ros/humble` 后应对 **默认内置** mapper 足够。扩展栈（nav2 / control / foxglove / apriltag）**不是**桥内置，不必为它们建 overlay。

无 ROS 环境时可用 `just check-ros2-shim`。`rclrs` 走 `ros_env::*`，crates.io 的 `ros-env` shim 是空的，本仓库用 [`third_party/ros-env-shim`](../../third_party/ros-env-shim) 通过 `[patch.crates-io]` 提供 **typed 字段桩**（按核心 mapper proto 生成）。我们自己的 `std_srvs` vendor 仍走系统 C typesupport，不依赖 rust IDL。

### Python（`rclpy`）

```bash
source /opt/ros/humble/setup.bash
just python-dev-ros2   # 或 just python-dev；需本机有 rclpy
```

实现目录：[`bindings/python/robot_bus/ros2_bridge/`](../../bindings/python/robot_bus/ros2_bridge/)（纯 Python）。ROS 侧是 `rclpy` 节点 + executor（后台线程 spin）；bus 侧是 `robot_bus.Node`。`ServiceClient` / `TopicPublisher` / `ActionClient` 进程内线程安全（每个句柄对 ZMQ socket 加 mutex）。`Ros2ToBus` 可在 rclpy executor 线程里直接调用。同一句柄上的并发调用会串行化。Mapper 按需 lazy import。

### C++（`rclcpp`）

- 链接 **`robot_bus_ros2_bridge`**（`ROBOT_BUS_HAS_ROS2`）；无此宏时 `build()` 抛错
- 本机构建：`just cpp-dev-ros2`（需先 `just gen-cpp` + source ROS）
- 包：`robot-bus-ros2-humble` / `robot-bus-ros2-jazzy`（**不** vendor `rcl`）

---

## 运行时

同进程同时持有 ROS 节点（rclrs / rclpy / rclcpp）和 robot-bus `Node`。主循环需推进两侧（`spin` / `spin_once`）：排空 ROS↔bus 队列并驱动 bus。

话题转换 / 解码 / 发出失败会丢掉该帧（桥继续转）。每座桥有原子计数，`drop_stats()` 返回整桥合计（`convert_fail` / `decode_fail` / `publish_fail`）。失败日志按路由限流（首次 + 每秒至多一条）。`build()` 会打印路由表。每条话题路由另有独立计数，经 `/robot_bus/bridges` 以 1 Hz 发到 console（侧栏 **BRIDGE**）。话题路由若在首次 `spin` 后 15s 仍从未收到样本，会 WARN 一次并往 `/robot_bus/events` 写「可能方向或 QoS 错了」。

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

## 常见问题

1. **未 source ROS** — 三端都会失败。
2. **C++ `ros2_available() == false`** — 未链 `robot_bus_ros2_bridge` / 装的是无桥包。
3. **Python `ros2_available() == False`** — 未安装或未 source 到 `rclpy`。
4. **Rust topic 登记了但跑不起来** — 缺对应 ROS typesupport，或发行版 rust IDL 未 source。

---

## 相关

- C++包与本地构建：[cpp-api.md](cpp-api.md)
- Python SDK：[python-api.md](python-api.md)
- API对比：[api-compare.md](api-compare.md)
