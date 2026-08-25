[English](../en/python-api.md) | 中文

# Python API 示例

```bash
pip install robot-bus
# 本地：just python-dev
# ROS 2 桥（rclpy）：source ROS 后 just python-dev-ros2；见 docs/zh/ros2-bridge.md
```

## Broker 启动

与 Rust 相同：先起 broker，再跑业务代码。

```bash
# 安装包后的 CLI
robot-bus-broker
robot-bus-broker --help
robot-bus-broker --api-listen 0.0.0.0:15570 --tcp-only
```

或进程内（关键字参数覆盖默认 bind / HWM / 心跳 / API）：

```python
import robot_bus

with robot_bus.RobotBusBroker.start(
    message_xsub_bind="tcp://127.0.0.1:15560",
    message_xpub_bind="tcp://127.0.0.1:15561",
    api_listen="0.0.0.0:15570",
    tcp_only=True,
) as broker:
    # broker.message_xsub_bind / message_xpub_bind / api_listen / console_listen
    # Web console: http://127.0.0.1:15570（no_console=True 可关；no_tank / no_docs 可隐藏侧栏菜单）
    pass
```

跨 broker（federation）与 CLI 同款字符串约定：

```python
with robot_bus.RobotBusBroker.start(
    broker_id="broker-a",
    message_peers=["tcp://10.0.0.2:15561"],          # peer XPUB；XSUB = 端口 - 1
    service_peers=["broker-b=tcp://10.0.0.2:15663"],  # 可选 id=
    action_peers=["broker-b=tcp://10.0.0.2:15665"],
    tcp_only=True,
    no_console=True,
) as broker:
    pass
```

### HTTP 发现（填地址，不选传输）

对已知 API 口请求 `GET /api/v1/discover`。传输仍手动指定（`tcp` / `ipc` / `inproc` / `ws`）；发现只填充位置：

```python
node = robot_bus.Node.discover(
    "talker", transport="tcp", api_url="http://127.0.0.1:15570")
# 可选：broker_id=...、timeout=...；UDP 组播发现已移除
```

多 broker 时可用 `broker_id=...` 过滤。

`Node(...)` **不会**等 broker。构造在 broker 未启动时也不抛错；TCP/WS 节点在后台重试 `GET /api/v1/discover`。可查 `node.connection_state`（`created` / `discovering` / `connecting` / `connected` / `reconnecting` / `shutdown`），或显式等待：

```python
node = robot_bus.Node("pilot")
if not node.wait_for_broker(timeout=5.0):
    raise SystemExit("broker not reachable")
node.add_on_connection_event(lambda old, new, reason: print(old, "->", new, reason))
```

`spin()` / `start()` 期间 broker 重启会自动重连。`create_*` 会短等 discover，仍未连上则报错。WebSocket 节点使用同一套 `connection_state`；Connected 表示 `/ws` 套接字已连通（不只是 HTTP discover）。

同进程 **inproc** 时必须共享 `Context`：

```python
ctx = robot_bus.Context()
with robot_bus.RobotBusBroker.start(context=ctx) as broker:
    node = robot_bus.Node.inproc_with_context(ctx, "pilot")
```

tcp / ipc / ws 不要求共享 Context。
默认端口与完整 CLI 选项见 [rust-api.md](rust-api.md)「Broker 启动」。

---

## 本地参数（Node）

本节点参数表（不经总线）。值类型为 Python `bool` / `int` / `float` / `str`；须先 `declare`，`set` 时类型须与声明一致。`get_parameter` / `declare_parameter` 返回 `{"name", "value"}`（对齐 ROS 2 Parameter，取值用 `["value"]`）。`list_parameters()` 返回 `{"names", "prefixes"}`；需要带值列表用 `list_all_parameters()`。支持 YAML 启动加载（扁平或 `ros__parameters` / `"/**"` 通配）。

```python
import robot_bus

node = robot_bus.Node("pilot")
node.declare_parameter("max_speed", 1.5)
node.declare_parameter("frame_id", "base_link")

print(node.get_parameter("max_speed")["value"])  # 1.5
node.set_parameter("max_speed", 2.0)
assert node.has_parameter("frame_id")
print(node.list_parameters())  # {"names": [...], "prefixes": [...]}
print(node.list_all_parameters())  # [{"name": "...", "value": ...}, ...]

node.load_parameters_from_yaml_str("""
ros__parameters:
  max_speed: 3.0
  enabled: true
""")
node.load_parameters_from_yaml_file("config/pilot.yaml")
```

---

## Message bus（Node + spin）

接近 ROS 2：`Node(...)` → `create_publisher` / `create_subscription` → `node.spin()`。单节点时无需手写 Executor（内部自动挂 `SingleThreadedExecutor`）。

仅走 WebSocket RPC 网关时用 `Node.ws` / `Node.ws_at`（或 `transport="ws"`）：可订阅、publish、调 service / action，不能当 server；见下文「WebSocket RPC 模式 Node」。

Python 主推 **typed**（创建时传入 protobuf 类，自动 `SerializeToString` / `ParseFromString`）；不传类型则仍为 raw bytes。底层与 Rust 一样走 opaque bytes（纯 Python 薄封装，因 PyO3 无法映射 Rust 泛型）。

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, imu: Imu):
    print(topic, imu.angular_velocity)

node = robot_bus.Node("pilot")
# 可选：Node(..., host=..., transport=..., message_xsub=..., message_xpub=...)

imu_pub = node.create_publisher("/robot1/imu", Imu)
node.create_subscription("/robot1/imu", on_imu, msg_type=Imu)

imu_pub.publish(
    Imu(
        angular_velocity=Vector3(x=0.0, y=0.0, z=0.1),
        linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8),
    )
)

# 阻塞直到 node.shutdown() 或 shutdown_handle().shutdown()
# node.spin()
```

完整可运行程序：[`examples/topic_imu/`](../../examples/topic_imu/)。

Raw bytes（与旧用法兼容）：

```python
imu_pub = node.create_publisher("/robot1/imu")  # → TopicPublisher
imu_pub.publish(imu.SerializeToString())

def on_raw(topic, payload: bytes):
    imu = Imu()
    imu.ParseFromString(payload)

node.create_subscription("/robot1/imu", on_raw)
```

### WebSocket RPC 模式 Node（客户端）

`Node.ws` / `Node.ws_at`（或 `Node(..., transport="ws", ws_url=...)`）经 broker WebSocket RPC 网关接入，不创建 ZMQ socket。

| 支持 | 不支持 |
|------|--------|
| `create_subscription` | `create_service` |
| `create_publisher` | `create_action_server` |
| `create_client` | 挂到 ZMQ Executor |
| `create_action_client` | |
| `create_timer`、`spin` / `shutdown` | |

```python
import robot_bus

node = robot_bus.Node.ws("web-client")
# 或 robot_bus.Node.ws_at("web-client", "http://127.0.0.1:15570")

pub = node.create_publisher("/robot1/cmd")
pub.publish(b"go")

def on_imu(topic, payload: bytes):
    print(topic, len(payload))

node.create_subscription("/robot1/imu", on_imu)

client = node.create_client("svc.echo")
reply = client.call(b"ping", timeout=2.0)

action = node.create_action_client("act.navigate")
goal = action.send_goal(
    b"goal",
    feedback_callback=lambda feedback: print("feedback", len(feedback)),
)  # GoalHandle 立即返回
result = goal.result(timeout=10.0)
# goal.cancel()  # best-effort，不表示服务端已确认

# 订阅回调需要 spin；action result 通过 GoalHandle 独立等待
# node.spin()
```

多节点共享或需多线程 service/action handler 时再显式用 Executor：

```python
executor = robot_bus.MultiThreadedExecutor(num_threads=4)
executor.add_node(node)
# executor.spin()
```

### Callback group

```python
group = node.create_callback_group(robot_bus.CallbackGroupType.Reentrant)
node.create_subscription("/robot1/imu", on_imu, callback_group=group)
node.create_timer(0.1, on_tick, callback_group=group)
node.create_service("echo", on_echo, callback_group=group)
node.create_action_server("navigate", on_goal, callback_group=group)
```

默认不传 `callback_group` 时使用节点自带的互斥组。`Reentrant` 需配合 `MultiThreadedExecutor` 才有实际并行。

### Service / Action（Node）

与 topic / timer 一样挂在 Node 上。可传 protobuf 类型做自动编解码，或省略走 raw bytes。Action client 采用 ROS 2 风格 `GoalHandle`：`send_goal` 立即返回，feedback 到达时实时调用回调，result 由 handle 独立等待。

```python
from robot_bus.std_srvs.srv.v1 import SetBoolRequest, SetBoolResponse
from robot_bus.example_interfaces.action.v1 import (
    FibonacciGoal,
    FibonacciFeedback,
    FibonacciResult,
)

def on_set_bool(req: SetBoolRequest) -> SetBoolResponse:
    return SetBoolResponse(success=True, message=f"set:{req.data}")

def on_fibonacci(goal: FibonacciGoal, context):
    seq = list(range(goal.order))
    context.publish_feedback(FibonacciFeedback(sequence=seq[:1]))
    return FibonacciResult(sequence=seq)

server_node = robot_bus.Node("worker")
cli_node = robot_bus.Node("caller")

server_node.create_service(
    "/set_bool", on_set_bool,
    request_type=SetBoolRequest, response_type=SetBoolResponse,
)
server_node.create_action_server(
    "/fibonacci", on_fibonacci,
    goal_type=FibonacciGoal,
    feedback_type=FibonacciFeedback,
    result_type=FibonacciResult,
)

svc = cli_node.create_client(
    "/set_bool",
    request_type=SetBoolRequest, response_type=SetBoolResponse,
)
# reply = svc.call(SetBoolRequest(data=True), timeout=5.0)

act = cli_node.create_action_client(
    "/fibonacci",
    goal_type=FibonacciGoal,
    feedback_type=FibonacciFeedback,
    result_type=FibonacciResult,
)
goal = act.send_goal(
    FibonacciGoal(order=5),
    feedback_callback=lambda feedback: print(feedback.sequence),
)
result = goal.result(timeout=10.0)
# goal.cancel()  # best-effort；不代表服务端已确认
# server_node.spin()
```

完整可运行程序：[`examples/service_set_bool/`](../../examples/service_set_bool/)、[`examples/action_fibonacci/`](../../examples/action_fibonacci/)。

Raw action 的 `feedback_callback(body: bytes)`、`ActionGoalHandle.result(timeout=None) -> bytes`；handle 还提供只读 `goal_id` / `action_name` 与 `cancel()`。`ActionClient.send_goal_and_wait(...)` 保留旧的批量消息列表用法。
Raw service：`handler(body: bytes) -> bytes` / `call(bytes)`。
endpoint 默认本机 broker；也可用 `Node(..., service_frontend=..., service_backend=..., action_backend=..., action_frontend=...)` 覆盖。

取消语义随 transport 不同：gRPC 取消对应 goal 的 server stream；ZMQ 发送显式 `CANCEL` 帧。两者都不承诺服务端确认取消。

### 定时器

与 topic 一样挂在 Node 上；回调由 `spin` / `spin_once` 驱动。

```python
import robot_bus

node = robot_bus.Node("timer_demo")

def on_tick():
    print("tick")

handle = node.create_timer(0.1, on_tick)  # 秒
# node.spin()

node.cancel_timer(handle)
```

### 非阻塞轮询

```python
import robot_bus

node = robot_bus.Node("poller")
node.create_subscription("/robot1/imu", lambda t, p: print(t))

while True:
    node.spin_once(timeout=0.1)  # 秒
    # 其它逻辑…
    break

node.shutdown()
```

### 从其它线程停止 spin

```python
import threading
import time
import robot_bus

node = robot_bus.Node("worker")
handle = node.shutdown_handle()

def stop_later():
    time.sleep(5)
    handle.shutdown()

threading.Thread(target=stop_later, daemon=True).start()
# node.spin()
```

---

## 与 Protobuf 配合

消息包挂在 `robot_bus.<pkg>.msg.v1`（与 Rust `robot_bus::<pkg>::msg::v1` 对齐）：

```python
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

imu = Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8))
payload = imu.SerializeToString()

imu2 = Imu()
imu2.ParseFromString(payload)
```

---

## ROS 2 桥（`rclpy`）

进程内 ROS ↔ bus 桥在 **`robot_bus.ros2_bridge`**，使用系统 **`rclpy`**（不经 Rust FFI）。完整契约见 [`ros2-bridge.md`](ros2-bridge.md)。

```bash
source /opt/ros/humble/setup.bash
just python-dev-ros2   # 安装 robot_bus；需本机有 rclpy
```

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

要点：

- 配置只走代码 `.mapper(具体对象)`；无 YAML、无类型名字符串挂路由
- 一期内置：`StdMsgsStringMapper`、`SensorMsgsImageMapper`、`TriggerServiceMapper`、`SetBoolServiceMapper`、`FibonacciActionMapper`
- `ros2_available()`：能否 `import rclpy`
- **自定义 service/action：可以**——先写对齐 ROS 的 bus `.proto` 并 `protoc`，再写 duck-typed mapper（`ros_srv_type` + `ros_req_to_bus` / `bus_req_to_ros` …），`.mapper(MyFoo())`；示例见 [ros2-bridge.md](ros2-bridge.md#用户自定义-service--action可以)
- `bus_discover(api_url="", timeout=0.0, broker_id="")` 与 C++/Rust 对齐（空 url / `timeout<=0` 用默认）

---

## 版本

```python
import robot_bus

print(robot_bus.__version__)
```

---

## 当前 Python API 一览

| 符号 | 说明 |
|------|------|
| `Node(name, host=..., transport=..., ws_url=..., message_xsub=..., …)` | 建节点；首次 `create_*` / `spin` 时自动挂 `SingleThreadedExecutor` |
| `Node.tcp` / `Node.ipc` / `Node.inproc` / `Node.inproc_with_context` / `Node.with_context` / `Node.ws` / `Node.ws_at` / `Node.discover` | 传输预设（推荐 `Context` + `with_context`；WebSocket RPC 网关为客户端模式；同进程 inproc 用 `inproc_with_context`；`discover` 只填地址） |
| `Node.declare_parameter` / `get_parameter` / `set_parameter` / `has_parameter` / `list_parameters` | 本节点本地参数（`bool` / `int` / `float` / `str`） |
| `Node.load_parameters_from_yaml_str` / `load_parameters_from_yaml_file` | 从 YAML 加载 / 覆盖参数 |
| `node.spin()` / `spin_once` / `shutdown` | 驱动回调（ROS 2 式简单路径） |
| `node.connection_state` / `wait_for_broker(timeout=None)` / `add_on_connection_event` | 与 broker 的会话：构造不阻塞；等待或观察 `connected` / `reconnecting` |
| `node.wait_for_message(topic, timeout=None)` | 等到一条消息或超时（返回 `bytes` / `None`） |
| `Context()` | 共享 ZMQ context（同进程 inproc 必需） |
| `SingleThreadedExecutor(context=None)` | 显式单线程执行器（多节点共享时用） |
| `MultiThreadedExecutor(num_threads=4, context=None)` | service/action handler 可并行 |
| `executor.add_node(node)` | 把节点挂到执行器（须在该节点尚未 auto-attach 之前） |
| `node.create_publisher(topic, msg_type=None, qos_depth=None)` | typed → `TypedTopicPublisher.publish(Message)`；省略类型 → raw；`qos_depth>0` → KeepLast HWM（WS 发布忽略） |
| `node.create_timer(period, callback)` → `TimerHandle` | 定时器（与 topic 一样挂在 Node） |
| `CallbackGroupType` / `create_callback_group` | `MutuallyExclusive` / `Reentrant` |
| `create_subscription(..., msg_type=, callback_group=, qos_depth=)` | typed：`callback(topic, Message)`；省略类型：`callback(topic, bytes)`；WS：`qos_depth` 为网关订阅队列深度 |
| `create_service(..., request_type=, response_type=)` | typed：`handler(Request) -> Response`；否则 raw bytes |
| `create_client(..., request_type=, response_type=)` | typed → `TypedServiceClient`；`service_is_ready` / `wait_for_service`（console workers） |
| `create_action_server(..., goal_type=, feedback_type=, result_type=)` | typed handler 通过 context 实时发布 feedback，并返回 result；否则 raw bytes |
| `create_action_client(..., goal_type=, feedback_type=, result_type=)` | typed → `TypedActionClient`；`wait_for_action_server`；`send_goal` → GoalHandle |
| `ActionGoalHandle` / `TypedActionGoalHandle` | goal 标识、action 名称、阻塞等待 result 与 best-effort cancel |
| `Publisher(endpoint=None)` | 低层连 XSUB（不经 Node） |
| `ros2_available()` | 能否 `import rclpy`（Python 原生桥） |
| `robot_bus.ros2_bridge.Ros2Bridge` / `Direction` / 内置 Mapper | 进程内 ROS 桥（**rclpy**）；见 [ros2-bridge.md](ros2-bridge.md) |
| `RobotBusBroker.start(...)` / `run_broker(...)` | 进程内启动三个 bus + WebSocket RPC API；同进程 inproc 传 `context`；peers 为 CLI 同款字符串列表 |
| `ShutdownHandle` / `TimerHandle` | spin 与定时器控制 |

WebSocket RPC 模式 Node 见上一节；底层网关 RPC 也可直接用 Rust tonic 客户端（[rust-api.md](rust-api.md)）。
