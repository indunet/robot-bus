# ROS 2 Bridge（`ros2_bridge`）用法

进程内把 **ROS 2** 与 **robot-bus** 的话题 / 服务 / Action 桥接在一起。实现位于 `src/ros2_bridge/`，公开 API：`robot_bus::ros2_bridge`（Cargo feature **`ros2`**）。

默认构建**不**启用该 feature，核心 SDK / crates.io / maturin 仍可免装 ROS。官方支持发行版：**Humble**、**Jazzy**。

## 前置条件

| 项 | 说明 |
|----|------|
| ROS 2 | 已安装并 `source /opt/ros/<distro>/setup.bash`（Humble 或 Jazzy） |
| Cargo | `robot-bus = { version = "…", features = ["ros2"] }` |
| Broker | 可达的 `robot_bus_broker`（tcp / ipc），或 `bus_discover` |
| C++ | 安装 `robot-bus-ros2-humble` / `robot-bus-ros2-jazzy`（**仅 Linux DEB**），包**不** vendor `rcl` |
| Python | 默认 wheel **不含** bridge；本地：`just python-dev-ros2`（需先 source ROS），运行时用 `robot_bus.ros2_available()` |

```bash
source /opt/ros/humble/setup.bash   # 或 jazzy
cargo run --bin robot_bus_broker    # 另开终端
```

本机构建 C++ 桥：`just cpp-dev-ros2`（需先 source ROS）。包选择与 stub 行为见 [cpp-api.md](cpp-api.md)。

## 方向（Direction）

| 值 | 含义 |
|----|------|
| `Ros2ToBus` / `ros2_to_bus` | ROS 侧发布 / 请求 → 写入 robot-bus |
| `BusToRos2` / `bus_to_ros2` | robot-bus → ROS 侧 |

话题 / 服务 / Action **只能二选一**（无 `both`）；默认都是 `Ros2ToBus`。

## 支持的类型

### 话题（与 `proto/` 对齐，200+ 种）

目录：`src/ros2_bridge/mappers/<pkg>/<msg>.rs`（**一种消息一个文件**）。共享字段读写在 `mappers/common.rs`（公开为 `robot_bus::ros2_bridge::mapper_support`）；内置服务 / Action 接线在 `mappers/service_bridges.rs`、`mappers/action_bridges.rs`。

话题类型注册表覆盖 [`proto/`](../proto) 里**所有**有 ROS 2 对应物的消息：`std_msgs`、`builtin_interfaces`、`geometry_msgs`、`sensor_msgs`、`nav_msgs`、`nav2_msgs`、`tf2_msgs`、`trajectory_msgs`、`diagnostic_msgs`、`shape_msgs`、`stereo_msgs`、`action_msgs`、`control_msgs`、`visualization_msgs`、`apriltag_msgs`、`unique_identifier_msgs`、`foxglove_msgs`。**不含** [`robot_bus_interface`](../proto/robot_bus_interface/)（总线内部，没有 ROS 包）。

链式 API **只认 `.mapper(具体类型)`**：内置 mapper 是公开 ZST（如 `StdMsgsStringMapper`）。自定义：Rust `impl TopicMapper`，或 C++ 继承 `robot_bus::TopicMapper` 后 `.mapper(std::make_shared<...>())`。YAML 仍用类型名字符串（仅内置）。`TopicMapper::type_name()` 由 mapper 自己提供，供 ROS 建 `DynamicMessage`。

**登记 ≠ 一定能跑**：桥用 `DynamicMessage` 反射，运行时还要求系统装了该类型的 typesupport（例如 `foxglove_msgs/msg/*` 需要 `foxglove_msgs` 包）。缺失时 `build()` 会在建 publisher / subscriber 时报错。

字段映射是宽松的：ROS 少某个字段就取 protobuf 默认值，多的字段忽略；数值宽度自动转换（ROS `int8[]` ↔ protobuf `repeated int32`、`uint8` ↔ `bool`）。

### 服务（内置 codec + 库接线）

| ROS 类型 | Mapper（类型标签） | 默认调用超时 |
|----------|-------------------|--------------|
| `std_srvs/srv/Trigger` | `TriggerServiceMapper` | 5s（可用 `.timeout(...)` 覆盖） |
| `std_srvs/srv/SetBool` | `SetBoolServiceMapper` | 5s |

`ServiceMapper` 是 **codec 标签**（提供 `type_name`）；库通过 typed 后端创建 ROS client/server。可用 `.timeout(Duration)` 覆盖默认超时。

### Action（内置 codec + 库接线）

| ROS 类型 | Mapper（类型标签） | 默认 goal 超时 |
|----------|-------------------|----------------|
| `example_interfaces/action/Fibonacci` | `FibonacciActionMapper` | 30s（可用 `.timeout(...)` 覆盖） |

rclrs **没有** dynamic service/action（升版本也不行）。因此：

- **内置**：`.mapper(TriggerServiceMapper)` 等即可，库负责接线
- **自定义 Rust**：可 `impl ServiceMapper` 并 **override `attach`**，在 `attach` 里用 `typed_service::attach_*` 风格自建 typed 实体；或调用 `typed_service::attach_trigger` 等 helper
- **C++ / Python 任意自定义 srv/action**：API 形状已与 Topic 对齐，但 **运行时任意 `type_name` 尚不可用**（需 Track B：dynamic RPC / rclrs wait-set）；目前只能用内置类型名

话题则可用 `DynamicMessage` + `mapper_support`，三端自定义 topic 已可用。

## 自定义类型（外挂，不改库源码）

每条 route / service / action **自己**挂 `.mapper(...)`（不是 builder 全局注册）：

| 方式 | 用途 |
|------|------|
| `.mapper(StdMsgsStringMapper)` 等内置 ZST | 库内已有类型（见 `mappers/`） |
| `.mapper(MyMapper)` | 自定义（topic 全语言；srv/action 见上） |

### 自定义话题

在**你的 crate**里：自备 ROS 包 + prost/protobuf，实现 `TopicMapper`，用 `mapper_support` 读写字段：

```rust
use robot_bus::ros2_bridge::{Direction, Ros2Bridge, TopicMapper, mapper_support};
use rclrs::DynamicMessage;

struct BatteryMapper;
impl TopicMapper for BatteryMapper {
    fn type_name(&self) -> &'static str { "my_robot_msgs/msg/BatteryState" }
    fn ros_to_bus(&self, msg: &DynamicMessage) -> robot_bus::Result<Vec<u8>> { /* ... */ }
    fn bus_to_ros(&self, payload: &[u8]) -> robot_bus::Result<DynamicMessage> { /* ... */ }
}

let mut bridge = Ros2Bridge::new("ros_bridge")
    .bus_tcp("localhost")
    .route("/battery", "/battery")
        .mapper(BatteryMapper)
        .direction(Direction::Ros2ToBus)
        .add()?
    .build()?;
```

运行时仍需能 `source` 到该 ROS 包的 typesupport。

### 自定义服务 / Action（Rust typed `attach`）

内置只需类型标签。高级 Rust 可 override `attach`（库已抽出接线到 `typed_service`）：

```rust
use robot_bus::ros2_bridge::{Direction, Ros2Bridge, ServiceMapper, ServiceWireContext, typed_service};

struct MyTrigger;
impl ServiceMapper for MyTrigger {
    fn type_name(&self) -> &'static str { "std_srvs/srv/Trigger" }
    // default attach → typed_service::attach_builtin_service
}

.service("/reset", "/reset")
    .mapper(MyTrigger) // or TriggerServiceMapper
    .timeout(std::time::Duration::from_secs(2))
    .direction(Direction::Ros2ToBus)
    .add()?
```

任意新 ROS srv/action 类型的跨语言 codec（对齐 Topic 的 DynMsg FFI）依赖 Track B。**Track B spike 结论：当前 rclrs 下不可行**（`add_to_wait_set` / `NodeHandle::rcl_node` 为 `pub(crate)`；详见 `ros2_bridge::dynamic_rpc::spike_summary()`）。未知类型会报「no typed … backend / unsupported …」。

C++ / Python：`.mapper` API **形状**已对齐 Topic（虚基类 / duck `type_name` + `.timeout(...)`）；自定义对象仅当 `type_name()` 为内置类型时走 typed 接线，convert 方法在 Track B 前不会被调用。

### YAML

YAML 的 `type:` **只能**引用内置类型。自定义必须用链式 `.mapper(...)`（可与 `builder_from_yaml` 混用：先加载 YAML，再 `.route(...).mapper(...).add()`）。

## Rust：链式 API

```rust
use robot_bus::ros2_bridge::{
    Direction, FibonacciActionMapper, Ros2Bridge, SensorMsgsImageMapper, SetBoolServiceMapper,
    StdMsgsStringMapper, TriggerServiceMapper,
};

fn main() -> robot_bus::Result<()> {
    let mut bridge = Ros2Bridge::new("ros_bridge")
        .bus_tcp("localhost")
        .route("/chatter", "/chatter")
            .mapper(StdMsgsStringMapper)
            .direction(Direction::Ros2ToBus)
            .add()?
        .route("/camera/image_raw", "/camera/image_raw")
            .mapper(SensorMsgsImageMapper)
            .direction(Direction::Ros2ToBus)
            .add()?
        .service("/reset", "/reset")
            .mapper(TriggerServiceMapper)
            .direction(Direction::Ros2ToBus)
            .add()?
        .service("/enable", "/enable")
            .mapper(SetBoolServiceMapper)
            .direction(Direction::BusToRos2)
            .add()?
        .action("/fibonacci", "/fibonacci")
            .mapper(FibonacciActionMapper)
            .direction(Direction::Ros2ToBus)
            .add()?
        .build()?;

    bridge.spin()?;
    Ok(())
}
```

### 连接 robot-bus

| 方法 | 作用 |
|------|------|
| `.bus_tcp(host)` | TCP 连指定主机（默认会走 discover 填端口） |
| `.bus_ipc()` / `.bus_ipc_at(dir)` | 本机 IPC |
| `.bus_discover(api_url)` | HTTP discover 后走 TCP |
| `.bus_discover_ex(api_url, timeout_secs, broker_id)` | 带超时 / broker 过滤 |

`route(ros_topic, bus_topic)` / `service(...)` / `action(...)` 的两个名字分别对应 ROS 侧与 bus 侧；可以同名，也可以 remap。

至少配置一条话题、服务或 Action，否则 `build()` 会失败。`build()` 会创建 ROS 节点与 bus 节点，并在后台线程 spin ROS executor。

## Rust / C++：YAML

```rust
let mut bridge = Ros2Bridge::from_yaml("bridge.yaml")?;
bridge.spin()?;
```

```cpp
auto bridge = robot_bus::Ros2Bridge::from_yaml("bridge.yaml");
bridge.spin();
```

### Schema

```yaml
robot_bus:
  transport: tcp          # tcp | ipc | discover
  host: localhost         # tcp 时使用
  # ipc_path: /tmp/robot_bus/...   # transport: ipc 时可选
  # discover:                       # transport: discover
  #   api_url: http://127.0.0.1:15570
  #   timeout: 3.0
  #   broker_id: null

routes:
  - ros_topic: /chatter
    bus_topic: /chatter
    type: std_msgs/msg/String
    direction: ros2_to_bus           # 默认 ros2_to_bus；禁止 both

services:
  - ros_service: /reset
    bus_service: /reset
    type: std_srvs/srv/Trigger
    direction: ros2_to_bus           # 默认 ros2_to_bus

actions:
  - ros_action: /fibonacci
    bus_action: /fibonacci
    type: example_interfaces/action/Fibonacci
    direction: ros2_to_bus           # 默认 ros2_to_bus
```

`routes` / `services` / `actions` 至少有一项非空。未写 `robot_bus` 时默认 `transport: tcp`、`host: localhost`。


## Python：链式 API

默认 `pip install robot-bus` / `just python-dev` **不**启用 `ros2`。需要先 source ROS，再：

```bash
just python-dev-ros2
# 或: cd bindings/python && maturin develop --features extension-module,grpc,ros2 --no-default-features
```

```python
import robot_bus
from robot_bus import (
    Direction, Ros2Bridge, StdMsgsStringMapper, SensorMsgsImageMapper,
    TriggerServiceMapper, FibonacciActionMapper,
)

assert robot_bus.ros2_available()

class BatteryMapper:
    def type_name(self):
        return "my_robot_msgs/msg/BatteryState"

    def ros_to_bus(self, msg):  # msg: DynMsg
        # msg.get_f64("percentage"); return protobuf bytes
        return b""

    def bus_to_ros(self, payload: bytes, msg) -> None:
        msg.set_f64("percentage", 1.0)

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .mapper(StdMsgsStringMapper)          # str 常量，等同 "std_msgs/msg/String"
    .direction(Direction.Ros2ToBus)
    .add()
    .route("/camera/image_raw", "/camera/image_raw")
    .mapper(SensorMsgsImageMapper)
    .add()
    .route("/battery", "/battery")
    .mapper(BatteryMapper())              # 自定义：type_name / ros_to_bus / bus_to_ros
    .add()
    .service("/reset", "/reset")
    .mapper(TriggerServiceMapper)
    .add()
    .action("/fibonacci", "/fibonacci")
    .mapper(FibonacciActionMapper)
    .add()
    .build()
)
bridge.spin()
# 或: Ros2Bridge.from_yaml("bridge.yaml")
```

Service / Action：Python / C++ 用内置类型标签（与 Topic 同形的 `.mapper(...)`），可用 `.timeout(secs)`。任意自定义 codec 的运行时接线需 Track B（当前 spike **blocked**）；对未知类型会报错。Rust 可 override `attach` 做 typed 后端。自定义 C++/Python 对象若 `type_name()` 命中内置则走 typed 接线（convert 方法暂不调用）。

## C++：链式 API

需安装带桥的 Linux 包并 source ROS；运行时可用 `robot_bus::ros2_available()` 判断库是否编译了 bridge。

```cpp
#include <robot_bus/Ros2Bridge.hpp>
#include <memory>
#include <vector>

// Custom C++ mapper (ROS DynMsg fields ↔ your bus protobuf bytes).
struct BatteryMapper : robot_bus::TopicMapper {
  const char *type_name() const override { return "my_robot_msgs/msg/BatteryState"; }
  std::vector<uint8_t> ros_to_bus(const robot_bus::DynMsg &msg) override {
    // e.g. msg.get_f64("percentage"); encode protobuf…
    return {};
  }
  void bus_to_ros(const uint8_t *payload, size_t len, robot_bus::DynMsg &msg) override {
    (void)payload; (void)len;
    msg.set_f64("percentage", 1.0);
  }
};

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .mapper(robot_bus::StdMsgsStringMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .route("/camera/image_raw", "/camera/image_raw")
    .mapper(robot_bus::SensorMsgsImageMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .route("/battery", "/battery")
    .mapper(std::make_shared<BatteryMapper>())
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .service("/reset", "/reset")
    .mapper(robot_bus::TriggerServiceMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .action("/fibonacci", "/fibonacci")
    .mapper(robot_bus::FibonacciActionMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .build();

bridge.spin();
```

YAML schema 与 Rust 相同。更多包与链接说明见 [cpp-api.md](cpp-api.md#ros-2-bridge-ros2bridge)。

## 运行时行为（简要）

- 桥进程同时持有一个 **rclrs** 节点和一个 **robot-bus** `Node`。
- ROS executor 在后台线程 spin；主线程 `spin` / `spin_once` 负责 bus 侧与 ROS→bus 出队发布。
- 服务 / Action 按方向在一侧当 server、另一侧当 client 转发；超时见上文常量。

## 常见问题

1. **`rclrs Context::default_from_env failed`** — 未 source ROS，或当前 shell 的 `AMENT_PREFIX_PATH` / 库路径不对。
2. **链接 / 找不到 typesupport** — 确认发行版与 feature 匹配；`CompressedVideo` 需安装 `foxglove_msgs`。
3. **连不上 broker** — 先起 `robot_bus_broker`，或改用 `bus_discover("http://127.0.0.1:15570")`。
4. **配了 `both`** — 话题 / 服务 / Action 都不支持，改成 `ros2_to_bus` 或 `bus_to_ros2`。
5. **C++ `ros2_available() == false`** — 装的是无桥的 `robot-bus` 包，请换 `robot-bus-ros2-*`。
6. **`unsupported ros2 bridge topic type "my_pkg/msg/Foo"`** — 内置表没有该类型。Rust：`impl TopicMapper` + `.mapper(...)`；C++：继承 `TopicMapper` + `.mapper(shared_ptr)`；Python：实现 `type_name` / `ros_to_bus` / `bus_to_ros` 的对象 + `.mapper(...)`（`DynMsg` 支持点路径）。YAML 无法单独挂自定义类型。
7. **`no typed service/action backend`** — srv/action 只有少量内置 codec；任意自定义需 Rust override `attach` 或等待 Track B（dynamic RPC）。C++/Python 目前只能用内置类型标签（`TriggerServiceMapper` 等）。
7. **Python `ros2_available() == False`** — 当前扩展未带 `ros2` feature；用 `just python-dev-ros2`（先 source ROS）重装。
8. **自定义类型 build 时报 typesupport / create DynamicMessage 失败** — ROS 包未安装或未 source；桥用反射，运行时必须能加载该消息的 typesupport。

## 相关

- 源码：[`src/ros2_bridge/`](../src/ros2_bridge/)
- C++ 包与本地构建：[cpp-api.md](cpp-api.md)
- 性能对标：[ros2-perf-report.md](ros2-perf-report.md)、`just perf-ros2`
