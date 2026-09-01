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
cargo run --bin robot_bus_broker    # 或已安装的 robot-bus-broker
```

| 语言 | 依赖 |
|------|------|
| 通用 | 可达的 broker（tcp / ipc / discover） |
| Rust | `robot-bus = { features = ["ros2"] }` + 下文「Rust overlay」 |
| Python | `robot_bus` + 系统 **`rclpy`**（`just python-dev` / `python-dev-ros2`） |
| C++ | `robot-bus-ros2-humble` 或 `…-jazzy`，或 `just cpp-dev-ros2`（`-DROBOT_BUS_ROS2=ON`） |

`ros2_available()`：Python 看能否 `import rclpy`；C++ 看是否以 `ROBOT_BUS_HAS_ROS2` 链了 `robot_bus_ros2_bridge`；Rust FFI / 默认 C ABI 恒为 false（桥不在 FFI 里）。

两端都写 **名字 + `TopicQos`**。`TopicQos.keep_last(n)` 之后必须 `.reliable()` 或 `.best_effort()`。ROS 端两者都行；**bus 端只能 `.best_effort()`**（没有 DDS reliability，depth 只变成 HWM）。ROS 端还可以再接 `.transient_local()`（默认 volatile），用来订 `/tf_static` 这类 latch 话题。

---

## 话题

ROS 在发、bus 侧要订：`.from_ros → .to_bus`。bus 在发、ROS 侧要订：`.from_bus → .to_ros`。挂上内置 mapper，`build()` 后 `spin()`。

```python
from robot_bus.ros2_bridge import Ros2Bridge, StdMsgsStringMapper, TopicQos

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", TopicQos.keep_last(10).reliable())
    .to_bus("/chatter", TopicQos.keep_last(8).best_effort())
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
    .from_ros("/chatter", TopicQos::keep_last(10).reliable())
    .to_bus("/chatter", TopicQos::keep_last(8).best_effort())
    .mapper(StdMsgsStringMapper)
    .add()?
    .build()?;
bridge.spin()?;
```

```cpp
#include <robot_bus/ros2_bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", robot_bus::TopicQos::keep_last(10).reliable())
    .to_bus("/chatter", robot_bus::TopicQos::keep_last(8).best_effort())
    .mapper(robot_bus::StdMsgsStringMapper{})
    .add()
    .build();
bridge.spin();
```

反向（bus → ROS）把链改成 `.from_bus("/chatter", …).to_ros("/chatter", …)`，其余一样。两边名字可以相同，也可以不同。

相机要对上 ROS 图上的 best-effort KeepLast(5) 时，两端都写 `keep_last(5).best_effort()`。

`/tf_static` 这类 latch 话题，ROS 端要加 `.transient_local()`，否则默认 volatile 订不到已经发过的样本：

```python
.from_ros("/tf_static", TopicQos.keep_last(1).reliable().transient_local())
.to_bus("/tf_static", TopicQos.keep_last(1).best_effort())
```

Rust / C++ 同样在 reliability 后面接 `.transient_local()`（C++ 若要改回默认，方法名是 `.durability_volatile()`，因为 `volatile` 是关键字）。bus 端没有 DDS durability，写了也会被忽略。

Topic mapper 三语言同一套目录（`proto/*/msg/v1`，约 214 个类型）：Rust `src/ros2_bridge/mappers/`，Python `robot_bus.ros2_bridge.mappers.<pkg>`，C++ `robot_bus/ros2_bridge/mappers/<pkg>/<msg>.hpp`。改 proto 后跑 `just gen-topic-mappers`。挂路由直接 `.mapper(GeometryMsgsPoseStampedMapper())` / `.mapper(GeometryMsgsPoseStampedMapper{})`。

| 示例 mapper | ROS 类型 |
|-------------|----------|
| `StdMsgsStringMapper` | `std_msgs/msg/String` |
| `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| `GeometryMsgsPoseStampedMapper` | `geometry_msgs/msg/PoseStamped` |

Rust 另有 `lookup_topic_mapper` / `registered_topic_types`。C++ Humble 默认没有的接口包（`nav2_msgs` / `control_msgs` / `apriltag_msgs` / `foxglove_msgs`）用 `find_package(... QUIET)`：找到才打开对应 mapper 的 ROS 转换。

### `.lazy()`（大流量 ROS → bus）

默认 eager：`build()` 立刻建 ROS subscription，ROS 图上能看到这座桥。相机、雷达等大流量才在 `from_ros → to_bus` 上加 `.lazy()`：没人订 bus 时，ROS 图上这座桥不是该 topic 的 subscriber。

```python
.from_ros("/camera/image", TopicQos.keep_last(5).best_effort())
.to_bus("/camera/image", TopicQos.keep_last(5).best_effort())
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
    def type_name(self) -> str:
        return "std_msgs/msg/String"

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

**C++**（`TypedTopicMapper` CRTP）：继承 `robot_bus::TypedTopicMapper<MyMapper, ros_msgs::msg::MyMsg>`，实现 `type_name` / `ros_to_bus` / `bus_to_ros`，见 [`ros2_bridge_typed.hpp`](../../bindings/cpp/include/robot_bus/ros2_bridge_typed.hpp)。

---

## 服务

先 `.service()`，再写两端名字 + QoS。ROS 对上 `services_default` 写 `keep_last(10).reliable()`；bus 常用 `keep_last(8).best_effort()`（depth → DEALER HWM）。默认超时 **5s**，要用 `.timeout(...)` 改。

ROS 提供服务、bus 客户端去调：`.from_ros → .to_bus`。bus 提供服务、ROS 客户端去调：`.from_bus → .to_ros`。

```python
.service()
.from_ros("/reset", TopicQos.keep_last(10).reliable())
.to_bus("/reset", TopicQos.keep_last(8).best_effort())
.mapper(TriggerServiceMapper())
.timeout(5.0)
.add()
```

```rust
.service()
    .from_ros("/reset", TopicQos::keep_last(10).reliable())
    .to_bus("/reset", TopicQos::keep_last(8).best_effort())
    .mapper(TriggerServiceMapper)
    .timeout(std::time::Duration::from_secs(5))
    .add()?
```

```cpp
.service()
    .from_ros("/reset", robot_bus::TopicQos::keep_last(10).reliable())
    .to_bus("/reset", robot_bus::TopicQos::keep_last(8).best_effort())
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
    .from_ros("/examples/add_two_ints", TopicQos.keep_last(10).reliable())
    .to_bus("/examples/add_two_ints", TopicQos.keep_last(8).best_effort())
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

// .service().from_ros("/examples/add_two_ints", TopicQos::keep_last(10).reliable())
//     .to_bus("/examples/add_two_ints", TopicQos::keep_last(8).best_effort())
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
.from_ros("/fibonacci", TopicQos.keep_last(10).reliable())
.to_bus("/fibonacci", TopicQos.keep_last(8).best_effort())
.mapper(FibonacciActionMapper())
.add()
```

```rust
.action()
    .from_ros("/fibonacci", TopicQos::keep_last(10).reliable())
    .to_bus("/fibonacci", TopicQos::keep_last(8).best_effort())
    .mapper(FibonacciActionMapper)
    .add()?
```

```cpp
.action()
    .from_ros("/fibonacci", robot_bus::TopicQos::keep_last(10).reliable())
    .to_bus("/fibonacci", robot_bus::TopicQos::keep_last(8).best_effort())
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

`feature = "ros2"` 还要一层 **ros2_rust** overlay：`AMENT_PREFIX_PATH` 里必须有 `rosidl_generator_rs` 生成的 `share/<pkg>/rust/`，覆盖 **mappers 里全部包**（不只是 `std_msgs` / `sensor_msgs`）。只 `source /opt/ros/humble` **编不出** typed 路径。`apt install ros-humble-sensor-msgs` 只提供 C typesupport，不含 rust IDL。

Humble 示例（在独立 workspace 里 `colcon build` 后 `source install/setup.bash`）：

```bash
mkdir -p ~/ros2_rust_ws/src && cd ~/ros2_rust_ws
git clone -b humble https://github.com/ros2/common_interfaces.git src/common_interfaces
git clone -b humble https://github.com/ros2/example_interfaces.git src/example_interfaces
git clone -b humble https://github.com/ros2/rcl_interfaces.git src/rcl_interfaces
git clone -b humble https://github.com/ros2/rosidl_core.git src/rosidl_core
git clone -b humble https://github.com/ros2/rosidl_defaults.git src/rosidl_defaults
git clone -b humble https://github.com/ros2/unique_identifier_msgs.git src/unique_identifier_msgs
git clone https://github.com/ros2-rust/rosidl_rust.git src/rosidl_rust
# Topic mappers also need rust IDL for these packages (same overlay workspace):
#   nav_msgs nav2_msgs geometry_msgs visualization_msgs tf2_msgs
#   diagnostic_msgs trajectory_msgs shape_msgs stereo_msgs
#   control_msgs foxglove_msgs apriltag_msgs action_msgs builtin_interfaces
source /opt/ros/humble/setup.bash
colcon build
source install/setup.bash
# 之后 cargo build --features ros2 才能看到 ros_env::<pkg>::msg
```

无 overlay 时可用 `just check-ros2-shim`。`rclrs` 0.8 走 `ros_env::*`，crates.io 的 `ros-env` shim 是空的，本仓库用 [`third_party/ros-env-shim`](../../third_party/ros-env-shim) 通过 `[patch.crates-io]` 提供 **typed 字段桩**（按 proto 生成，不是 DynamicMessage 退路）。我们自己的 `std_srvs` vendor 仍走系统 C typesupport，不依赖 rust IDL。

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

---

## 常见问题

1. **未 source ROS** — 三端都会失败。
2. **C++ `ros2_available() == false`** — 未链 `robot_bus_ros2_bridge` / 装的是无桥包。
3. **Python `ros2_available() == False`** — 未安装或未 source 到 `rclpy`。
4. **Rust topic 登记了但跑不起来** — 缺对应 ROS typesupport（如 `foxglove_msgs`）。

---

## 相关

- C++包与本地构建：[cpp-api.md](cpp-api.md)
- Python SDK：[python-api.md](python-api.md)
- API对比：[api-compare.md](api-compare.md)
