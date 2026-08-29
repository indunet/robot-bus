[English](../en/ros2-bridge.md) | 中文

# ROS2 Bridge（`ros2_bridge`）

进程内把 **ROS2** 与 **robot-bus** 桥在一起：Topic / Service / Action。

## 架构：三语言各自原生

| 语言 | ROS客户端 | 入口 | 说明 |
|------|------------|------|------|
| **Rust** | `rclrs` | `robot_bus::ros2_bridge`（Cargo feature **`ros2`**） | Topic / service / action都走 typed `attach`（`TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper`） |
| **Python** | **`rclpy`** | `robot_bus.ros2_bridge` | 纯 Python，**不经** Rust FFI / `rclrs` |
| **C++** | **`rclcpp`** | `<robot_bus/ros2_bridge.hpp>` + `robot_bus_ros2_bridge` | 原生 C++，**不经** Rust FFI / `rclrs` |

```text
Python:  rclpy  ──mapper──► robot_bus.Node
C++:     rclcpp ──mapper──► robot_bus::Node
Rust:    rclrs  ──mapper──► robot_bus::Node
```

**为什么分语言：** Topic / service / action都要用编译期具体类型（`create_subscription<T>` / `create_service<T>`等）。若 C++/Python只把类型名交给 Rust，`T`对不上。因此各语言在本侧用具体类型建 ROS实体，再用本语言 bus `Node`转发。

| 支持 | 不支持 |
|------|--------|
| Topic / Service / Action | YAML配置桥 |
| 代码里 `.mapper(具体对象)` | 用类型名字符串 lookup挂路由 |
| 用户自定义 mapper（本语言具体类型） | 跨语言「只传字符串」的万能桥 |

官方发行版：**Humble**、**Jazzy**。

---

## 前置条件

```bash
source /opt/ros/humble/setup.bash   # 或 jazzy
cargo run --bin robot_bus_broker    # 或已安装的 robot-bus-broker
```

**Rust `feature = "ros2"`** 用 crates.io **`rclrs` 0.7**。typed消息走已发布的 **`ros-env` 0.2**：它扫 `AMENT_PREFIX_PATH`上的 `share/<pkg>/rust/`，再导出成 `ros_env::sensor_msgs::msg::Image`。当前 Humble apt里 `common_interfaces`（含 `sensor_msgs`）**已经带 rust IDL**，只 `source /opt/ros/humble`就能编 Image / String这类。完整 mapper注册表还要 `nav2_msgs` / `control_msgs` / `apriltag_msgs`等 distro里没有的包，缺的才需要 overlay，见下文「Rust消息」。

| 语言 | 依赖 |
|------|------|
| 通用 | 可达的 broker（tcp / ipc / discover） |
| Rust | `robot-bus = { features = ["ros2"] }` + 已 source的 ROS（typed消息见下文） |
| Python | `robot_bus` + 系统 **`rclpy`**（`just python-dev` / `python-dev-ros2`） |
| C++ | `robot-bus-ros2-humble`或 `…-jazzy`，或 `just cpp-dev-ros2`（`-DROBOT_BUS_ROS2=ON`，链 **rclcpp**） |

`ros2_available()`：

- **Python**：能否 `import rclpy`
- **C++**：是否以 `ROBOT_BUS_HAS_ROS2`链接了 `robot_bus_ros2_bridge`
- **Rust FFI / 默认 C ABI**：恒为 false（桥不在 FFI里）

---

## 最小例子：把一条 ROS topic接到 bus上

**Mapper** 是字段对照表（ROS对象 ↔ protobuf），不是传输。`.mapper(StdMsgsStringMapper())`只告诉桥「这条路由用哪张表」。库负责建 ROS订阅/发布、protobuf编解码、往 bus收发。

方向、QoS都写在**每一条 `.route()`上**，不是写在 `Ros2Bridge.new(...)`上。方向只有两个：`Ros2ToBus`（默认：ROS出、bus进）或 `BusToRos2`（bus出、ROS进），**没有** `both`。要双向就挂两条路由。

默认方向（ROS → bus）实际走的是：

```text
ROS侧有人 publish std_msgs/String
        │
        ▼
   /examples/chatter     ← .route的第一个参数（ROS名）
        │
        ▼  桥在 ROS图上是这个 topic的 subscriber
StdMsgsStringMapper把 ROS String转成 protobuf
        │
        ▼  桥在 bus上是 publisher
   /examples/chatter     ← .route的第二个参数（bus名，可以和 ROS名不同）
        │
        ▼
broker转给 bus上订了这个名字的进程
```

下面这条链和仓库示例 [`examples/ros2_bridge/python/builtin.py`](../../examples/ros2_bridge/python/builtin.py) 同一套名字。括号里是**一行表达式**，从左往右读。

```python
import robot_bus
from robot_bus.ros2_bridge import (
    Direction,
    Ros2Bridge,
    StdMsgsStringMapper,
    TriggerServiceMapper,
)

assert robot_bus.ros2_available()

bridge = (
    Ros2Bridge.new("ros_bridge")        # 进程内：ROS节点名 = bus节点名
    .bus_tcp("localhost")               # 连本机 broker；也可 .bus_ipc() / .bus_discover(...)
    .route("/examples/chatter", "/examples/chatter")  # (ROS名, bus名)
    .mapper(StdMsgsStringMapper())      # 必须是对象，不能写 "std_msgs/msg/String"
    .direction(Direction.Ros2ToBus)     # 可省略，默认就是 ROS→bus
    .add()                              # 提交这一条 topic；后面可以再 .route / .service
    .service("/examples/reset", "/examples/reset")
    .mapper(TriggerServiceMapper())
    .timeout(3.0)                       # 秒；默认 5s
    .add()
    .build()
)
bridge.spin()                           # 同时推进 ROS executor和 bus
```

Rust / C++同一条链，语法见文末；可运行文件在 [`examples/ros2_bridge/`](../../examples/ros2_bridge/)。**没有** `from_yaml`，**没有** `add_route(..., "std_msgs/msg/String", ...)`。

### 怎么跑、怎么确认通了

四个终端都先 `source /opt/ros/humble/setup.bash`（或 jazzy）。

**终端 1 — broker**

```bash
robot-bus-broker    # 或 cargo run --bin robot_bus_broker
```

**终端 2 — 桥**（用仓库示例，topic是 `/examples/chatter`）

```bash
python3 examples/ros2_bridge/python/builtin.py
```

`ros2 topic info /examples/chatter`应能看到这座桥是 subscriber。

**终端 3 — ROS侧发**

```bash
ros2 topic pub /examples/chatter std_msgs/msg/String "{data: hello}"
```

**终端 4 — bus侧收**（应打印 `hello`）

```python
import robot_bus
from robot_bus.std_msgs.msg.v1 import String

node = robot_bus.Node("peek")
node.create_subscription(
    "/examples/chatter",
    lambda topic, msg: print(msg.data),
    msg_type=String,
)
node.spin()
```

反方向（bus → ROS）：同一条 `.route`加上 `.direction(Direction.BusToRos2)`。那时桥在 ROS上是 publisher、在 bus上是 subscriber；用 bus的 `create_publisher`发，用 `ros2 topic echo /examples/chatter`收。

---

## 在同一条 `.route()`上改 QoS

QoS **跟 `.mapper()`写在同一条路由上**，不写在 `Ros2Bridge.new(...)`上。Service / action没有这些 helper（仍用 ROS默认 service QoS）。

不写 helper时：ROS用各语言默认（Python/C++ `QoS(10)` reliable，Rust `topics_default()`）；bus用节点默认 HWM。

在最小例子那条 `/examples/chatter`上加一行即可：

```python
.route("/examples/chatter", "/examples/chatter")
.mapper(StdMsgsStringMapper())
.qos_depth(20)      # ROS KeepLast(20) 并且 bus HWM=20（同一个 n，不能两边分开）
.add()
```

多条路由时，每条自己写 QoS / 方向 / lazy：

```python
from robot_bus.ros2_bridge import (
    Direction,
    Ros2Bridge,
    SensorMsgsImageMapper,
    StdMsgsStringMapper,
)

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .mapper(StdMsgsStringMapper())
    .qos_depth(20)
    .add()
    .route("/camera/image", "/camera/image")
    .mapper(SensorMsgsImageMapper())
    .sensor_data()                      # ROS SensorDataQoS + bus depth 5
    .lazy()                             # 可选：bus上没人订时，ROS图上看不到这座桥
    .add()
    .route("/from_bus", "/from_bus")
    .mapper(StdMsgsStringMapper())
    .direction(Direction.BusToRos2)     # 这条是 bus→ROS
    .qos_depth(10)
    .best_effort()                      # 只改 ROS reliability；bus本来就是 best-effort
    .add()
    .build()
)
```

Rust / C++方法名相同（Rust的 `.add()?`带问号）。完整程序见文末和 `examples/ros2_bridge/`。

| helper | ROS | bus |
|--------|-----|-----|
| `.qos_depth(n)` | KeepLast(n) | `QosProfile::keep_last(n)`（HWM） |
| `.best_effort()` | reliability = best effort | 不变（bus没有 DDS reliability） |
| `.sensor_data()` | `SensorDataQoS`（best-effort KeepLast 5） | depth 5 |

现在**不能**「ROS depth=10、bus HWM=50」各写一套，也没有 durability / deadline等完整 ROS QoS。相机用 `.sensor_data().lazy()`；不要把 Image内置 mapper的默认改成 SensorDataQoS。

### `.lazy()`（仅 ROS2→bus）

默认 **eager**：`.add()`之后 `build()`立刻建 ROS subscription，`ros2 topic info`能看到这座桥。大流量（相机、雷达）才 `.lazy()`：bus上没人订时，桥不出现在该 ROS topic的 subscriber列表里。

- **`.lazy()`无参。** 不要 `.lazy(true)`。
- **只允许 `Ros2ToBus`。** 配在 `BusToRos2`上 `.add()`会报错。Service / action没有 `.lazy()`。
- **无 console的 broker**（`--no-console`）：lazy **降级为 eager**（没有 demand信号）。
- 需求只数 `kind == subscriber`。裸 `Subscriber`（不经 `Node`）以及关掉 topology的 WebSocket **打不开** lazy。

broker在 subscriber注册/注销时往 `/robot_bus/topic_demand`发 [`TopicDemand`](../../proto/robot_bus_interfaces/msg/v1/console_status.proto)；桥启动时再读 `/robot_bus/topics`，避免订阅者先于桥启动时 lazy一直关着。

C++只 override `attach`、把实体塞进 `keep_alive`的自定义 mapper **不支持** `.lazy()`。请用 `TypedTopicMapper`。

链式 API一览（每条 `.route` / `.service` / `.action`以 `.add()`结尾）：

```text
Ros2Bridge.new(name)
  .bus_tcp(...) | .bus_ipc() | .bus_discover(...)
  .route(ros, bus).mapper(...).direction(...).qos_depth(n)|.best_effort()|.sensor_data().lazy().add()
  .service(ros, bus).mapper(...).timeout(...).direction(...).add()
  .action(ros, bus).mapper(...).timeout(...).direction(...).add()
  .build()
  .spin()
```

默认超时：service **5s**，action goal **30s**。

### 一期内置 mapper（对象，不是字符串）

| 种类 | Mapper | ROS类型 |
|------|--------|----------|
| Topic | `StdMsgsStringMapper` | `std_msgs/msg/String` |
| Topic | `SensorMsgsImageMapper` | `sensor_msgs/msg/Image` |
| Service | `TriggerServiceMapper` | `std_srvs/srv/Trigger` |
| Service | `SetBoolServiceMapper` | `std_srvs/srv/SetBool` |
| Action | `FibonacciActionMapper` | `example_interfaces/action/Fibonacci` |

Rust另有完整 topic mapper注册表（`src/ros2_bridge/mappers/`），挂路由仍须 `.mapper(具体类型)`；`lookup_topic_mapper` / `registered_topic_types`仅自省，不是挂路由入口。

---

## 用户自定义 mapper：可以

**可以。** 先写 **bus protobuf**（字段对齐 ROS `.msg` / `.srv` / `.action`），`protoc`生成本语言 stubs，再只写 **字段 ↔ protobuf转换**；库负责订阅/发布/service接线。typed API接受任意 protobuf消息类，不必放进 robot-bus仓库。

| | 行不行 |
|--|--------|
| Python：duck-typed convert方法 + `.mapper(MyFoo())` | **行** |
| Rust：`impl TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` | **行** |
| C++：`TypedTopicMapper` / `TypedServiceMapper` CRTP + `.mapper(shared_ptr)` | **行**（需 `ROBOT_BUS_HAS_ROS2`） |
| 只写 YAML / 类型名字符串 | **不行** |

高级：仍可直接 override `ServiceMapper::attach` / `ActionMapper::attach`（特殊 QoS等）。

下面以 ROS `example_interfaces/srv/AddTwoInts`为例（与工程内自有 `my_pkg/srv/AddTwoInts`写法相同），从 proto写到挂桥。

**可运行示例**（Python / Rust / C++）：[`examples/ros2_bridge/`](../../examples/ros2_bridge/)
— `builtin`为 phase-1内置 mapper，`custom_add_two_ints`为本自定义流程。

### 1. 定义 bus protobuf

ROS侧已有（Humble/Jazzy自带 `example_interfaces`）：

```text
# example_interfaces/srv/AddTwoInts.srv
int64 a
int64 b
---
int64 sum
```

Bus侧按同样字段写 `.proto`（建议 ROS风格包路径 + `v1`）。本仓库已提供
[`proto/example_interfaces/srv/v1/add_two_ints.proto`](../../proto/example_interfaces/srv/v1/add_two_ints.proto)：

```protobuf
syntax = "proto3";
package example_interfaces.srv.v1;

// Equivalent to ROS2 `example_interfaces/srv/AddTwoInts`.
message AddTwoIntsRequest {
  int64 a = 1;
  int64 b = 2;
}

message AddTwoIntsResponse {
  int64 sum = 1;
}
```

若是**工程内自有类型**（如 `my_pkg`），自行生成 stubs：

```bash
# Python
protoc --python_out=. --pyi_out=. my_pkg/srv/v1/add_two_ints.proto

# C++
protoc --cpp_out=. my_pkg/srv/v1/add_two_ints.proto
```

Rust在 `build.rs`里：

```rust
prost_build::compile_protos(
    &["proto/my_pkg/srv/v1/add_two_ints.proto"],
    &["proto"],
)?;
```

若要把类型贡献进本仓库内置集合：文件放到 [`proto/`](../../proto/) 对应目录，再 `just gen-*`。

### Python：自定义 Service mapper

桥调用：`ros_srv_type()`、`ros_req_to_bus` / `bus_req_to_ros`、`ros_resp_to_bus` / `bus_resp_to_ros`。

完整可运行文件：[`examples/ros2_bridge/python/custom_add_two_ints.py`](../../examples/ros2_bridge/python/custom_add_two_ints.py)。

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

Action：同一套流程——proto里写 Goal / Feedback / Result三个 message，再实现 `ros_action_type()` + 六向转换（见 [`mappers/fibonacci.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/fibonacci.py)）。

### Rust：自定义 Service（`TypedServiceMapper`）

`include!`生成代码后，用 prost类型编解码。
可运行：[`examples/ros2_bridge/rust/custom_add_two_ints.rs`](../../examples/ros2_bridge/rust/custom_add_two_ints.rs)。

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

Action：`impl TypedActionMapper`（`type Ros = …` + goal/feedback/result六向转换）。库内 `wire_typed_*`负责接线。

自定义 topic：`impl TypedTopicMapper`（关联 `Ros` IDL类型与 `Bus` protobuf类型），只写字段转换；库负责 `create_subscription` / `create_publisher`。

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

### C++：自定义 Service（`TypedServiceMapper` CRTP）

内置仍用 ZST：`.mapper(TriggerServiceMapper{})`。自定义继承 CRTP，只写转换；库自动 `attach` / `retain`。
可运行：[`examples/ros2_bridge/cpp/custom_add_two_ints.cpp`](../../examples/ros2_bridge/cpp/custom_add_two_ints.cpp)。

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

Topic / Action：同一套「先 proto再 mapper」。`TypedTopicMapper` / `TypedActionMapper`见 [`ros2_bridge_typed.hpp`](../../bindings/cpp/include/robot_bus/ros2_bridge_typed.hpp)。

---

## Rust（`rclrs`）

与上文「最小例子」同一条链。QoS / lazy写在 `.route()`上。完整可运行：[`examples/ros2_bridge/rust/builtin.rs`](../../examples/ros2_bridge/rust/builtin.rs)。

```rust
use robot_bus::ros2_bridge::{
    Direction, Ros2Bridge, StdMsgsStringMapper, TriggerServiceMapper,
};

fn main() -> robot_bus::Result<()> {
    let mut bridge = Ros2Bridge::new("ros_bridge")
        .bus_tcp("localhost")
        .route("/examples/chatter", "/examples/chatter")
        .mapper(StdMsgsStringMapper)
        .direction(Direction::Ros2ToBus)
        .add()?
        .service("/examples/reset", "/examples/reset")
        .mapper(TriggerServiceMapper)
        .timeout(std::time::Duration::from_secs(3))
        .add()?
        .build()?;
    bridge.spin()?;
    Ok(())
}
```

- 自定义 topic：`impl TypedTopicMapper`（关联 `Ros` / `Bus`，`ros_to_bus` / `bus_to_ros`做字段直拷；库负责订阅/发布）
- 自定义 service/action：`TypedServiceMapper` / `TypedActionMapper`（见上文「用户自定义」）
- 模块：`typed_service`（`wire_typed_*` / `attach_*`）

### Rust消息（`ros-env` + ament rust IDL）

客户端是 crates.io **`rclrs` 0.7**。消息类型来自 **`ros-env` 0.2** 对 `share/<pkg>/rust/`的再导出，不是 rclrs自带的。

Humble上 `ros-humble-sensor-msgs`等包已经带 `share/sensor_msgs/rust/`（含 `msg::Image`）。`source /opt/ros/humble`之后，`ros_env`能看到这些 crate。仓库里整份 topic mapper注册表还依赖若干 **apt默认没有** 的包（`nav2_msgs`、`control_msgs`、`apriltag_msgs`）；要用完整注册表，把缺的接口包放进 overlay workspace再 `colcon build`：

```bash
mkdir -p ~/ros2_rust_ws/src && cd ~/ros2_rust_ws
# 只补 distro没有 rust IDL的包，例如：
git clone -b humble https://github.com/ros-navigation/nav2_msgs.git src/nav2_msgs
# control_msgs / apriltag_msgs同理
git clone https://github.com/ros2-rust/rosidl_rust.git src/rosidl_rust
source /opt/ros/humble/setup.bash
colcon build
source install/setup.bash
# 之后 cargo build --features ros2才能看到 ros_env::<pkg>::msg
```

无 overlay时可用 `just check-ros2-shim`。crates.io的 `ros-env`在 `use_ros_shim`下是空的，本仓库用 [`third_party/ros-env-shim`](../../third_party/ros-env-shim) 通过 `[patch.crates-io]`提供 **typed字段桩**（按 proto生成，不是 DynamicMessage退路）。我们自己的 `std_srvs` vendor仍走系统 C typesupport，不依赖 rust IDL。

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

assert robot_bus.ros2_available()  # import rclpy成功

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .route("/examples/chatter", "/examples/chatter")
    .mapper(StdMsgsStringMapper())
    .direction(Direction.Ros2ToBus)
    .add()
    .service("/examples/reset", "/examples/reset")
    .mapper(TriggerServiceMapper())
    .add()
    .build()
)
bridge.spin()
```

- ROS侧：`rclpy`节点 + executor（后台线程 spin）
- Bus侧：`robot_bus.Node`（raw / typed protobuf）
- **线程：** `ServiceClient` / `TopicPublisher` / `ActionClient`为进程内线程安全（每个句柄对 ZMQ socket加 mutex，满足 `Send + Sync`）。`Ros2ToBus`可在 rclpy executor线程里直接调用（与 C++侧对 client加 `std::mutex`同思路）。同一句柄上的并发调用会串行化。
- 自定义 mapper：见上文「用户自定义」；内置参考 [`mappers/trigger.py`](../../bindings/python/robot_bus/ros2_bridge/mappers/trigger.py)
- Mapper按需 lazy import（依赖对应 ROS消息包与 protobuf）

---

## C++（`rclcpp`）

```cpp
#include <robot_bus/ros2_bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .route("/examples/chatter", "/examples/chatter")
    .mapper(robot_bus::StdMsgsStringMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .service("/examples/reset", "/examples/reset")
    .mapper(robot_bus::TriggerServiceMapper{})
    .add()
    .build();

bridge.spin();
```

- 链接 **`robot_bus_ros2_bridge`**（`ROBOT_BUS_HAS_ROS2`）；无此宏时 `build()`抛错
- 本机构建：`just cpp-dev-ros2`（需先 `just gen-cpp` + source ROS）
- 包：`robot-bus-ros2-humble` / `robot-bus-ros2-jazzy`（**不** vendor `rcl`）
- 内置 ZST + `.mapper(std::shared_ptr<…Mapper>)`自定义；见上文「用户自定义」

---

## 运行时

同进程同时持有：

1. ROS节点（rclrs / rclpy / rclcpp）
2. robot-bus `Node`

主循环需推进两侧（`spin` / `spin_once`）；各语言实现细节不同，语义一致：排空 ROS↔bus队列并驱动 bus。

---

## 常见问题

1. **未 source ROS** — 三端都会失败。
2. **YAML配桥** — 不支持；代码里挂 mapper。
3. **只传类型名字符串** — 不支持挂路由；传具体 mapper对象。
4. **想跨语言万能动态 srv** — 不做；在目标语言写自定义 mapper。
5. **C++ `ros2_available() == false`** — 未链 `robot_bus_ros2_bridge` / 装的是无桥包。
6. **Python `ros2_available() == False`** — 未安装或未 source到 `rclpy`。
7. **Rust topic登记了但跑不起来** — 缺对应 ROS typesupport（如 `foxglove_msgs`）。

---

## 相关

- C++包与本地构建：[cpp-api.md](cpp-api.md)
- Python SDK：[python-api.md](python-api.md)
- API对比：[api-compare.md](api-compare.md)
