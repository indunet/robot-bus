# Python API 示例

安装：

```bash
pip install robot-bus
```

本地开发（需 [maturin](https://www.maturin.rs/)）：

```bash
maturin develop --features extension-module
```

模块名：`robot_bus`。当前 Python 绑定覆盖 **message bus** 与 **内嵌 broker**；service / action 请用 Rust 或后续版本。

---

## 启动 broker

### 命令行

```bash
robot-bus-broker
```

### 进程内

```python
import robot_bus

with robot_bus.RobotBusBroker.start() as broker:
    print("message XPUB:", broker.message_xpub_bind)
    print("service frontend:", broker.service_frontend_bind)
    # 业务代码…
# 离开 with 自动 stop
```

阻塞运行（等同命令行，Ctrl+C 退出）：

```python
import robot_bus

robot_bus.run_broker()
```

---

## 端点辅助函数

```python
import robot_bus

xsub = robot_bus.message_xsub_endpoint()              # 默认 localhost + tcp
xpub = robot_bus.message_xpub_endpoint("127.0.0.1", "tcp")
```

默认连本机 broker：XSUB `15560`，XPUB `15561`。

---

## Message bus（Node + spin）

接近 ROS 2 的 `create_publisher` / `create_subscription` / `spin`；payload 用与 Rust `robot_bus::msgs` 同 schema 的 protobuf（此处标准 `Imu`）：

```python
import robot_bus
# 与 Rust `robot_bus::msgs::sensor_msgs::msg::v1::Imu` 同 schema（需自行生成 pb）
from sensor_msgs_pb2 import Imu
from geometry_msgs_pb2 import Vector3

def on_imu(topic, payload):
    imu = Imu()
    imu.ParseFromString(payload)
    print(topic, imu.angular_velocity)

node = robot_bus.Node("pilot", namespace="robot1")
node.create_publisher(robot_bus.message_xsub_endpoint())
node.create_subscription("imu", on_imu, robot_bus.message_xpub_endpoint())

imu = Imu(
    angular_velocity=Vector3(x=0.0, y=0.0, z=0.1),
    linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8),
)
node.publish("imu", imu.SerializeToString())  # 实际 topic: robot1/imu

# 阻塞直到 node.shutdown() 或 shutdown_handle().shutdown()
# node.spin()
```

### 定时器

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
node.create_subscription("imu", lambda t, p: print(t), robot_bus.message_xpub_endpoint())

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

## 命名空间

```python
import robot_bus

node = robot_bus.Node("pilot", namespace="robot1")

assert node.name == "pilot"
assert node.namespace == "robot1"
assert node.fully_qualified_name() == "robot1/pilot"
assert node.resolve_name("imu") == "robot1/imu"
assert node.resolve_name("/imu") == "/imu"  # 绝对名不变
```

---

## 与 Protobuf 配合

Python 侧自行用 `protobuf` 包编码（与 `robot_bus::msgs` 同 schema）。上面示例用的是标准 `sensor_msgs/msg/v1/Imu`；其它消息同理，例如 `Twist`：

```python
# 假设已生成 geometry_msgs_pb2.Twist
from geometry_msgs_pb2 import Twist, Vector3

twist = Twist(linear=Vector3(x=1.0, y=0.0, z=0.0))
node.publish("cmd_vel", twist.SerializeToString())
```

仓库内 ROS 风格 proto 在 `proto/<pkg>/{msg|srv|grpc}/v1/`，Rust 侧由 `robot_bus::msgs` 提供类型。

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
| `Node(name, namespace=None)` | 命名空间 + publisher / subscription / timer / spin |
| `Publisher(endpoint=None)` | 连 XSUB，发布 topic（也可经 `Node.publish`） |
| `RobotBusBroker.start()` | 进程内启动三个 bus |
| `run_broker()` | 阻塞 CLI 入口 |
| `message_xsub_endpoint(host, transport)` | 发布端点 |
| `message_xpub_endpoint(host, transport)` | 订阅端点 |
| `ShutdownHandle` / `TimerHandle` | spin 与定时器控制 |

Service / Action、gRPC 网关目前仅 Rust 侧提供；见 [rust-api.md](rust-api.md)。
