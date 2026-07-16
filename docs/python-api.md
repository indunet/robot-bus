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

连接由 `Node` 管理（默认 `localhost` + `tcp`）；`create_publisher` / `create_subscription` 不再传 endpoint。

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

## Message bus（Node + spin）

接近 ROS 2 的 `create_publisher` / `create_subscription` / `spin`。回调收到 `bytes`，用同 schema 的 protobuf 解析（标准 `Imu` 随 `robot-bus` 打包）：

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, payload):
    imu = Imu()
    imu.ParseFromString(payload)
    print(topic, imu.angular_velocity)

node = robot_bus.Node("pilot", namespace="robot1")
# 可选：host=..., transport=..., message_xsub=..., message_xpub=...
node.create_publisher()
node.create_subscription("imu", on_imu)

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
node.create_subscription("imu", lambda t, p: print(t))

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

`pip install robot-bus` 后消息类型已在 `robot_bus` 命名空间下（与 Rust `robot_bus::<pkg>::…` 同 proto schema）。例如 `Twist`：

```python
from robot_bus.geometry_msgs.msg.v1 import Twist, Vector3

twist = Twist(linear=Vector3(x=1.0, y=0.0, z=0.0))
node.publish("cmd_vel", twist.SerializeToString())
```

仓库内 ROS 风格 proto 在 `proto/<pkg>/{msg|srv|grpc}/v1/`。Python 生成物在 `python/robot_bus/<pkg>/…`（由 `scripts/generate_python_msgs.py` 生成并随 wheel 发布）。Typed 订阅（回调直接收解码后的消息）目前仅 Rust：`create_subscription_typed`。

说明：

- 路径形如 `robot_bus.sensor_msgs.msg.v1`，**不占用** ROS 顶层包名 `sensor_msgs`
- 编码是 protobuf；与 ROS IDL/CDR **字节不互通**（除非另做 bridge）
- 生成文件名 `*_pb2.py` 是 protoc 惯例，不表示 proto2

---

## 端点辅助函数

低层 `Publisher` / 手工拼地址时仍可用：

```python
import robot_bus

xsub = robot_bus.message_xsub_endpoint()              # 默认 localhost + tcp
xpub = robot_bus.message_xpub_endpoint("127.0.0.1", "tcp")
```

默认本机 broker：XSUB `15560`，XPUB `15561`。

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
| `Node(name, namespace=None, host=..., transport=..., message_xsub=..., message_xpub=...)` | 连接配置 + publisher / subscription / timer / spin |
| `Publisher(endpoint=None)` | 低层连 XSUB（也可经 `Node.publish`） |
| `RobotBusBroker.start()` | 进程内启动三个 bus |
| `run_broker()` | 阻塞 CLI 入口 |
| `message_xsub_endpoint(host, transport)` | 发布端点辅助 |
| `message_xpub_endpoint(host, transport)` | 订阅端点辅助 |
| `ShutdownHandle` / `TimerHandle` | spin 与定时器控制 |

Service / Action、gRPC 网关目前仅 Rust 侧提供；见 [rust-api.md](rust-api.md)。
