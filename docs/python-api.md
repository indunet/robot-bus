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
```

或进程内：

```python
import robot_bus

with robot_bus.RobotBusBroker.start() as broker:
    # broker.message_xsub_bind / message_xpub_bind 等
    pass
```

默认端口见 [rust-api.md](rust-api.md)「Broker 启动」。

---

## Message bus（Executor + Node + spin）

接近 ROS 2：先建 Executor，再 `create_node`，然后 `create_publisher` / `create_subscription` / `spin`。回调收到 `bytes`，用同 schema 的 protobuf 解析（标准 `Imu` 随 `robot-bus` 打包）：

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, payload):
    imu = Imu()
    imu.ParseFromString(payload)
    print(topic, imu.angular_velocity)

executor = robot_bus.SingleThreadedExecutor()
# 可选：create_node(..., host=..., transport=..., message_xsub=..., message_xpub=...)
node = executor.create_node("pilot")
node.create_publisher()
node.create_subscription("/robot1/imu", on_imu)

imu = Imu(
    angular_velocity=Vector3(x=0.0, y=0.0, z=0.1),
    linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8),
)
node.publish("/robot1/imu", imu.SerializeToString())

# 阻塞直到 executor.shutdown() 或 shutdown_handle().shutdown()
# executor.spin()
```

多线程 service/action handler：

```python
executor = robot_bus.MultiThreadedExecutor(num_threads=4)
node = executor.create_node("pilot")
```

### 定时器

```python
import robot_bus

executor = robot_bus.SingleThreadedExecutor()
node = executor.create_node("timer_demo")

def on_tick():
    print("tick")

handle = node.create_timer(0.1, on_tick)  # 秒
# executor.spin()

node.cancel_timer(handle)
```

### 非阻塞轮询

```python
import robot_bus

executor = robot_bus.SingleThreadedExecutor()
node = executor.create_node("poller")
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

executor = robot_bus.SingleThreadedExecutor()
node = executor.create_node("worker")
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
| `SingleThreadedExecutor()` | 单线程执行器；`create_node` + `spin` |
| `MultiThreadedExecutor(num_threads=4)` | service/action handler 可并行 |
| `executor.create_node(name, host=..., …)` | 创建 Node；topic 用全路径 |
| `Publisher(endpoint=None)` | 低层连 XSUB（也可经 `Node.publish`） |
| `RobotBusBroker.start()` | 进程内启动三个 bus |
| `run_broker()` | 阻塞 CLI 入口 |
| `message_xsub_endpoint(host, transport)` | 发布端点辅助 |
| `message_xpub_endpoint(host, transport)` | 订阅端点辅助 |
| `ShutdownHandle` / `TimerHandle` | spin 与定时器控制 |

Service / Action、gRPC 网关目前仅 Rust 侧提供；见 [rust-api.md](rust-api.md)。
