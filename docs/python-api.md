# Python API 示例

```bash
pip install robot-bus
# 本地：maturin develop --features extension-module
```

## Broker 启动

与 Rust 相同：先起 broker，再跑业务代码。

```bash
# 安装包后的 CLI
robot-bus-broker
robot-bus-broker --help
robot-bus-broker --grpc-listen 0.0.0.0:15770 --tcp-only
```

或进程内（关键字参数覆盖默认 bind / HWM / 心跳 / gRPC）：

```python
import robot_bus

with robot_bus.RobotBusBroker.start(
    message_xsub_bind="tcp://127.0.0.1:15560",
    message_xpub_bind="tcp://127.0.0.1:15561",
    grpc_listen="0.0.0.0:15770",
    tcp_only=True,
) as broker:
    # broker.message_xsub_bind / message_xpub_bind / grpc_listen 等
    pass
```

默认端口与完整 CLI 选项见 [rust-api.md](rust-api.md)「Broker 启动」。

---

## Message bus（Executor + Node + spin）

接近 ROS 2：`Node(...)` → `executor.add_node(node)` → `create_publisher(topic)` → `publisher.publish(...)`。Python 侧为 raw bytes（自行 `SerializeToString` / `ParseFromString`）；Rust 侧主推 typed `create_publisher::<Imu>` / `create_subscription::<Imu, _>`。

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, payload):
    imu = Imu()
    imu.ParseFromString(payload)
    print(topic, imu.angular_velocity)

node = robot_bus.Node("pilot")
# 可选：Node(..., host=..., transport=..., message_xsub=..., message_xpub=...)
executor = robot_bus.SingleThreadedExecutor()
executor.add_node(node)

imu_pub = node.create_publisher("/robot1/imu")
node.create_subscription("/robot1/imu", on_imu)

imu = Imu(
    angular_velocity=Vector3(x=0.0, y=0.0, z=0.1),
    linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8),
)
imu_pub.publish(imu.SerializeToString())

# 阻塞直到 executor.shutdown() 或 shutdown_handle().shutdown()
# executor.spin()
```

多线程 service/action handler：

```python
executor = robot_bus.MultiThreadedExecutor(num_threads=4)
executor.add_node(node)
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

与 topic / timer 一样挂在 Node 上：server 用 `create_service` / `create_action_server`，client 用 `create_client` / `create_action_client`。

```python
def on_echo(body: bytes) -> bytes:
    return b"echo:" + body

def on_goal(payload: bytes):
    # 返回 [(phase, body), ...]；phase 一般为 "FEEDBACK" / "RESULT"
    return [
        ("FEEDBACK", b"step-1"),
        ("RESULT", b"done:" + payload),
    ]

server_node = robot_bus.Node("worker")
cli_node = robot_bus.Node("caller")
executor = robot_bus.SingleThreadedExecutor()
executor.add_node(server_node)

server_node.create_service("echo", on_echo)
server_node.create_action_server("navigate", on_goal)

svc = cli_node.create_client("echo")
# reply = svc.call(b"ping", timeout=5.0)

act = cli_node.create_action_client("navigate")
# messages = act.send_goal(b"go", timeout=10.0)
# executor.spin()
```

endpoint 默认本机 broker；也可用 `Node(..., service_frontend=..., service_backend=..., action_backend=..., action_frontend=...)` 覆盖。

### 定时器

与 topic 一样挂在 Node 上：先 `executor.add_node(node)`，再 `node.create_timer`；回调由 `spin` / `spin_once` 驱动。

```python
import robot_bus

node = robot_bus.Node("timer_demo")
executor = robot_bus.SingleThreadedExecutor()
executor.add_node(node)

def on_tick():
    print("tick")

handle = node.create_timer(0.1, on_tick)  # 秒
# executor.spin()

node.cancel_timer(handle)
```

### 非阻塞轮询

```python
import robot_bus

node = robot_bus.Node("poller")
executor = robot_bus.SingleThreadedExecutor()
executor.add_node(node)
node.create_subscription("/robot1/imu", lambda t, p: print(t))

while True:
    executor.spin_once(timeout=0.1)  # 秒
    # 其它逻辑…
    break

executor.shutdown()
```

### 从其它线程停止 spin

```python
import threading
import time
import robot_bus

node = robot_bus.Node("worker")
executor = robot_bus.SingleThreadedExecutor()
executor.add_node(node)
handle = executor.shutdown_handle()

def stop_later():
    time.sleep(5)
    handle.shutdown()

threading.Thread(target=stop_later, daemon=True).start()
# executor.spin()
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

## 版本

```python
import robot_bus

print(robot_bus.__version__)
```

---

## 当前 Python API 一览

| 符号 | 说明 |
|------|------|
| `Node(name, host=..., message_xsub=..., service_frontend=..., service_backend=..., …)` | 建节点（尚未挂到 executor） |
| `SingleThreadedExecutor()` | 单线程执行器；`add_node` + `spin` |
| `MultiThreadedExecutor(num_threads=4)` | service/action handler 可并行 |
| `executor.add_node(node)` | 把节点挂到执行器（ROS 2 同款） |
| `node.create_publisher(topic)` → `TopicPublisher` | raw publisher；`publish(bytes)` |
| `node.create_timer(period, callback)` → `TimerHandle` | 定时器（与 topic 一样挂在 Node） |
| `CallbackGroupType` / `create_callback_group` | `MutuallyExclusive` / `Reentrant` |
| `create_subscription(..., callback_group=)` | raw 订阅；`callback(topic, bytes)` |
| `create_service(name, handler, …)` | service server；`handler(body) -> bytes` |
| `create_client(name)` → `ServiceClient` | service client；`call(body, timeout=…)` |
| `create_action_server(name, handler, …)` | action server；handler 返回 `[(phase, bytes), ...]` |
| `create_action_client(name)` → `ActionClient` | action client；`send_goal` / `cancel` |
| `Publisher(endpoint=None)` | 低层连 XSUB（不经 Node） |
| `RobotBusBroker.start()` | 进程内启动三个 bus + gRPC |
| `run_broker()` | 阻塞 CLI 入口 |
| `ShutdownHandle` / `TimerHandle` | spin 与定时器控制 |

gRPC 网关目前仅 Rust 侧提供；见 [rust-api.md](rust-api.md)。
