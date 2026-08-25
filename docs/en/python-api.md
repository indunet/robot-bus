English | [中文](../zh/python-api.md)

# Python API examples

```bash
pip install robot-bus
# Local: just python-dev
# ROS 2 bridge (rclpy): source ROS then just python-dev-ros2; see docs/en/ros2-bridge.md
```

## Broker startup

Same as Rust: start the broker first, then run application code.

```bash
# CLI after installing the package
robot-bus-broker
robot-bus-broker --help
robot-bus-broker --api-listen 0.0.0.0:15570 --tcp-only
```

Or in-process (keyword arguments override default bind / HWM / heartbeat / API):

```python
import robot_bus

with robot_bus.RobotBusBroker.start(
    message_xsub_bind="tcp://127.0.0.1:15560",
    message_xpub_bind="tcp://127.0.0.1:15561",
    api_listen="0.0.0.0:15570",
    tcp_only=True,
) as broker:
    # broker.message_xsub_bind / message_xpub_bind / api_listen / console_listen
    # Web console: http://127.0.0.1:15570  (pass no_console=True to disable; no_tank / no_docs hide sidebar entries)
    pass
```

Cross-broker (federation) uses the same string conventions as the CLI:

```python
with robot_bus.RobotBusBroker.start(
    broker_id="broker-a",
    message_peers=["tcp://10.0.0.2:15561"],          # peer XPUB; XSUB = port - 1
    service_peers=["broker-b=tcp://10.0.0.2:15663"],  # optional id=
    action_peers=["broker-b=tcp://10.0.0.2:15665"],
    tcp_only=True,
    no_console=True,
) as broker:
    pass
```

### HTTP discovery (fills in addresses, does not choose transport)

Request `GET /api/v1/discover` on a known API base URL. Transport is still specified manually (`tcp` / `ipc` / `inproc` / `ws`); discovery only fills in locations:

```python
node = robot_bus.Node.discover(
    "talker", transport="tcp", api_url="http://127.0.0.1:15570")
# Optional: broker_id=..., timeout=...; UDP multicast discovery has been removed
```

When multiple brokers are reachable, pass `broker_id=...` to filter.

`Node(...)` does **not** wait for the broker. Construction never raises on a missing broker; TCP/WS nodes retry `GET /api/v1/discover` in the background. Check `node.connection_state` (`created` / `discovering` / `connecting` / `connected` / `reconnecting` / `shutdown`) or wait:

```python
node = robot_bus.Node("pilot")
if not node.wait_for_broker(timeout=5.0):
    raise SystemExit("broker not reachable")
node.add_on_connection_event(lambda old, new, reason: print(old, "->", new, reason))
```

`spin()` / `start()` keep retrying if the broker restarts. `create_*` waits a few seconds for discover, then raises if still disconnected. WebSocket nodes use the same `connection_state` values; Connected means the `/ws` socket is up (not merely HTTP discover).

Same-process **inproc** requires a shared `Context`:

```python
ctx = robot_bus.Context()
with robot_bus.RobotBusBroker.start(context=ctx) as broker:
    node = robot_bus.Node.inproc_with_context(ctx, "pilot")
```

tcp / ipc / ws do not require a shared Context.
Default ports and full CLI options: see [rust-api.md](rust-api.md) “Broker startup”.

---

## Local parameters (Node)

Parameter table for this node (not on the bus). Value types are Python `bool` / `int` / `float` / `str`; you must `declare` first, and `set` types must match the declaration. `get_parameter` / `declare_parameter` return `{"name", "value"}` (ROS 2 Parameter shape; read with `["value"]`). `list_parameters()` returns `{"names", "prefixes"}`; use `list_all_parameters()` for name+value list. Supports YAML load at startup (flat or `ros__parameters` / `"/**"` wildcard).

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

## Message bus (Node + spin)

Close to ROS 2: `Node(...)` → `create_publisher` / `create_subscription` → `node.spin()`. With a single node you do not need to hand-write an Executor (it auto-attaches `SingleThreadedExecutor` internally).

For the WebSocket RPC gateway only, use `Node.ws` / `Node.ws_at` (or `transport="ws"`): you can subscribe, publish, and call service / action, but cannot act as a server; see “WebSocket RPC mode Node” below.

Python recommends **typed** usage (pass a protobuf class at creation for automatic `SerializeToString` / `ParseFromString`); omit the type for raw bytes. Under the hood it is the same as Rust with opaque bytes (thin Python wrapper; PyO3 cannot map Rust generics).

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, imu: Imu):
    print(topic, imu.angular_velocity)

node = robot_bus.Node("pilot")
# Optional: Node(..., host=..., transport=..., message_xsub=..., message_xpub=...)

imu_pub = node.create_publisher("/robot1/imu", Imu)
node.create_subscription("/robot1/imu", on_imu, msg_type=Imu)

imu_pub.publish(
    Imu(
        angular_velocity=Vector3(x=0.0, y=0.0, z=0.1),
        linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8),
    )
)

# Blocks until node.shutdown() or shutdown_handle().shutdown()
# node.spin()
```

Full runnable programs: [`examples/topic_imu/`](../../examples/topic_imu/).

Raw bytes (compatible with older usage):

```python
imu_pub = node.create_publisher("/robot1/imu")  # → TopicPublisher
imu_pub.publish(imu.SerializeToString())

def on_raw(topic, payload: bytes):
    imu = Imu()
    imu.ParseFromString(payload)

node.create_subscription("/robot1/imu", on_raw)
```

### WebSocket RPC mode Node (client)

`Node.ws` / `Node.ws_at` (or `Node(..., transport="ws", ws_url=...)`) connect via the broker WebSocket RPC gateway and do not create ZMQ sockets.

| Supported | Not supported |
|------|--------|
| `create_subscription` | `create_service` |
| `create_publisher` | `create_action_server` |
| `create_client` | attach to ZMQ Executor |
| `create_action_client` | |
| `create_timer`, `spin` / `shutdown` | |

```python
import robot_bus

node = robot_bus.Node.ws("web-client")
# or robot_bus.Node.ws_at("web-client", "http://127.0.0.1:15570")

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
)  # GoalHandle returns immediately
result = goal.result(timeout=10.0)
# goal.cancel()  # best-effort; does not mean the server confirmed

# Subscriptions need spin; action result waits independently via GoalHandle
# node.spin()
```

Use an Executor explicitly when sharing multiple nodes or needing multi-threaded service/action handlers:

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

By default, without `callback_group`, the node’s mutually exclusive group is used. `Reentrant` requires `MultiThreadedExecutor` for actual parallelism.

### Service / Action (Node)

Same as topic / timer: attached to the Node. Pass protobuf types for automatic encode/decode, or omit for raw bytes. The action client uses ROS 2–style `GoalHandle`: `send_goal` returns immediately, the feedback callback runs as feedback arrives, and `result` is waited on independently via the handle.

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
# goal.cancel()  # best-effort; does not mean the server confirmed
# server_node.spin()
```

Full runnable programs: [`examples/service_set_bool/`](../../examples/service_set_bool/), [`examples/action_fibonacci/`](../../examples/action_fibonacci/).

Raw action: `feedback_callback(body: bytes)`, `ActionGoalHandle.result(timeout=None) -> bytes`; the handle also exposes read-only `goal_id` / `action_name` and `cancel()`. `ActionClient.send_goal_and_wait(...)` keeps the older batch message list usage.
Raw service: `handler(body: bytes) -> bytes` / `call(bytes)`.
The endpoint defaults to the local broker; override with `Node(..., service_frontend=..., service_backend=..., action_backend=..., action_frontend=...)`.

Cancel semantics vary by transport: gRPC cancel targets the goal’s server stream; ZMQ sends an explicit `CANCEL` frame. Neither guarantees the server acknowledged cancel.

### Timers

Same as topic: attached to the Node; callbacks are driven by `spin` / `spin_once`.

```python
import robot_bus

node = robot_bus.Node("timer_demo")

def on_tick():
    print("tick")

handle = node.create_timer(0.1, on_tick)  # seconds
# node.spin()

node.cancel_timer(handle)
```

### Non-blocking poll

```python
import robot_bus

node = robot_bus.Node("poller")
node.create_subscription("/robot1/imu", lambda t, p: print(t))

while True:
    node.spin_once(timeout=0.1)  # seconds
    # other logic…
    break

node.shutdown()
```

### Stop spin from another thread

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

## Working with Protobuf

Message packages live under `robot_bus.<pkg>.msg.v1` (aligned with Rust `robot_bus::<pkg>::msg::v1`):

```python
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

imu = Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8))
payload = imu.SerializeToString()

imu2 = Imu()
imu2.ParseFromString(payload)
```

---

## ROS 2 bridge (`rclpy`)

The in-process ROS ↔ bus bridge is in **`robot_bus.ros2_bridge`**, using system **`rclpy`** (not Rust FFI). Full contract: [`ros2-bridge.md`](ros2-bridge.md).

```bash
source /opt/ros/humble/setup.bash
just python-dev-ros2   # installs robot_bus; requires rclpy on the host
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

Key points:

- Configuration is code-only via `.mapper(concrete object)`; no YAML, no type name strings on routes
- Built-in (phase 1): `StdMsgsStringMapper`, `SensorMsgsImageMapper`, `TriggerServiceMapper`, `SetBoolServiceMapper`, `FibonacciActionMapper`
- `ros2_available()`: whether `import rclpy` succeeds
- **Custom service/action: yes** — write a bus `.proto` aligned with the ROS type and `protoc` it, then a duck-typed mapper (`ros_srv_type` + `ros_req_to_bus` / `bus_req_to_ros` …) and `.mapper(MyFoo())`; example in [ros2-bridge.md](ros2-bridge.md#user-defined-service--action-yes)
- `bus_discover(api_url="", timeout=0.0, broker_id="")` aligns with C++/Rust (empty url / `timeout<=0` uses defaults)

---

## Version

```python
import robot_bus

print(robot_bus.__version__)
```

---

## Current Python API reference

| Symbol | Description |
|------|------|
| `Node(name, host=..., transport=..., ws_url=..., message_xsub=..., …)` | Create a node; auto-attaches `SingleThreadedExecutor` on first `create_*` / `spin` |
| `Node.tcp` / `Node.ipc` / `Node.inproc` / `Node.inproc_with_context` / `Node.with_context` / `Node.ws` / `Node.ws_at` / `Node.discover` | Transport presets (prefer `Context` + `with_context`; WebSocket RPC gateway is client mode; same-process inproc uses `inproc_with_context`; `discover` only fills addresses) |
| `Node.declare_parameter` / `get_parameter` / `set_parameter` / `has_parameter` / `list_parameters` | Local node parameters (`bool` / `int` / `float` / `str`) |
| `Node.load_parameters_from_yaml_str` / `load_parameters_from_yaml_file` | Load / override parameters from YAML |
| `node.spin()` / `spin_once` / `shutdown` | Drive callbacks (ROS 2–style simple path) |
| `node.connection_state` / `wait_for_broker(timeout=None)` / `add_on_connection_event` | Broker link: construct does not block; wait or observe `connected` / `reconnecting` |
| `node.wait_for_message(topic, timeout=None)` | Wait for one message or timeout (`bytes` / `None`) |
| `Context()` | Shared ZMQ context (required for same-process inproc) |
| `SingleThreadedExecutor(context=None)` | Explicit single-threaded executor (for shared multi-node use) |
| `MultiThreadedExecutor(num_threads=4, context=None)` | Parallel service/action handlers |
| `executor.add_node(node)` | Attach node to executor (must be before auto-attach on that node) |
| `node.create_publisher(topic, msg_type=None, qos_depth=None)` | typed → `TypedTopicPublisher`; omit type → raw; `qos_depth>0` → KeepLast HWM (ignored on WS publish) |
| `node.create_timer(period, callback)` → `TimerHandle` | Timer (attached to Node like topic) |
| `CallbackGroupType` / `create_callback_group` | `MutuallyExclusive` / `Reentrant` |
| `create_subscription(..., msg_type=, callback_group=, qos_depth=)` | typed: `callback(topic, Message)`; omit type: `callback(topic, bytes)`; WS: `qos_depth` sizes the gateway subscribe queue |
| `create_service(..., request_type=, response_type=)` | typed: `handler(Request) -> Response`; otherwise raw bytes |
| `create_client(..., request_type=, response_type=)` | typed → `TypedServiceClient`; `service_is_ready` / `wait_for_service` (console workers) |
| `create_action_server(..., goal_type=, feedback_type=, result_type=)` | typed handler publishes feedback in real time via context and returns result; otherwise raw bytes |
| `create_action_client(..., goal_type=, feedback_type=, result_type=)` | typed → `TypedActionClient`; `wait_for_action_server`; `send_goal` → GoalHandle |
| `ActionGoalHandle` / `TypedActionGoalHandle` | Goal id, action name, blocking wait for result, best-effort cancel |
| `Publisher(endpoint=None)` | Low-level XSUB connection (without Node) |
| `ros2_available()` | Whether `import rclpy` succeeds (native Python bridge) |
| `robot_bus.ros2_bridge.Ros2Bridge` / `Direction` / built-in Mapper | In-process ROS bridge (**rclpy**); see [ros2-bridge.md](ros2-bridge.md) |
| `RobotBusBroker.start(...)` / `run_broker(...)` | Start three buses + WebSocket RPC API in-process; pass `context` for same-process inproc; peers use CLI-style string lists |
| `ShutdownHandle` / `TimerHandle` | Spin and timer control |

WebSocket RPC mode Node: see previous section; low-level gateway RPC can also use the Rust tonic client directly ([rust-api.md](rust-api.md)).
