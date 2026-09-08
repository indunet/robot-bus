English | [中文](../zh/python-api.md)

# Python API

```bash
pip install robot-bus
# Local: just python-dev
# ROS 2 bridge (rclpy): source ROS then just python-dev-ros2; see docs/en/ros2-bridge.md
```

## Broker startup

Same as Rust: **prefer starting the broker from your program**, then run application code. The CLI is for demos, multi-process bring-up, or a standalone long-running broker.

In-process (keyword arguments override default bind / HWM / heartbeat / API):

```python
import robot_bus

with robot_bus.RobotBusBroker.start(
    message_xsub_bind="tcp://127.0.0.1:15580",
    message_xpub_bind="tcp://127.0.0.1:15581",
    api_listen="0.0.0.0:15560",
    tcp_only=True,
) as broker:
    # broker.message_xsub_bind / message_xpub_bind / api_listen / console_listen
    # Web console: http://127.0.0.1:15560  (pass no_console=True to disable; no_tank / no_docs hide sidebar entries)
    pass
```

Use the CLI when you need a standalone process:

```bash
python -m robot_bus.broker
python -m robot_bus.broker --help
python -m robot_bus.broker --api-listen 0.0.0.0:15560 --tcp-only
```

Cross-broker (federation) uses the same string conventions as the CLI:

```python
with robot_bus.RobotBusBroker.start(
    broker_id="broker-a",
    message_peers=["tcp://10.0.0.2:15581"],          # peer XPUB; XSUB = port - 1
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
    "talker", transport="tcp", api_url="http://127.0.0.1:15560")
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

`spin()` / `start()` keep retrying if the broker restarts. `create_*` waits a few seconds for discover, then raises if still disconnected. WebSocket nodes use the same `connection_state` values; Connected means the `/ws-rpc` socket is up (not merely HTTP discover).

Same-process **inproc** requires a shared `Context`:

```python
ctx = robot_bus.Context()
with robot_bus.RobotBusBroker.start(context=ctx) as broker:
    node = robot_bus.Node.inproc_with_context(ctx, "pilot")
```

tcp / ipc / ws do not require a shared Context.
Standalone process: `python -m robot_bus.broker --help`. Default ports and full CLI options: see [rust-api.md](rust-api.md) “Broker startup”.

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

For WebSocket communication, use `Node.ws` / `Node.ws_at` (or `transport="ws"`): you can subscribe, publish, and call service / action, but cannot act as a server; see “WebSocket RPC mode Node” below.

Python recommends **typed** usage (pass a protobuf class at creation for automatic `SerializeToString` / `ParseFromString`); omit the type for raw bytes. Under the hood it is the same as Rust with opaque bytes (thin Python wrapper; PyO3 cannot map Rust generics).

Start a broker as shown in the [README quick start](../../README.md). This complete program runs independently. `start()` drives callbacks on a Rust background thread; do not move a Python Node to another Python thread.

<!-- runnable: topic -->
```python
import os
import time
from threading import Event

import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

api_url = os.environ.get("ROBOT_BUS_API_URL", "http://127.0.0.1:15560")
node = robot_bus.Node.discover("pilot", transport="tcp", api_url=api_url)
received = Event()

def on_imu(imu: Imu):
    if not received.is_set():
        print(f"acceleration.z: {imu.linear_acceleration.z}")
    received.set()

try:
    node.create_subscription("/robot1/imu", on_imu, msg_type=Imu)
    pub = node.create_publisher("/robot1/imu", Imu)
    node.start()
    deadline = time.monotonic() + 5.0
    while not received.is_set() and time.monotonic() < deadline:
        pub.publish(Imu(linear_acceleration=Vector3(z=9.8)))
        received.wait(0.05)
    if not received.is_set():
        raise TimeoutError("No IMU received within 5 seconds")
finally:
    node.shutdown()
    node.stop()
    node.wait()
```

Expected output includes: `acceleration.z: 9.8`.


Full runnable programs: [`examples/topic_imu/`](../../examples/topic_imu/).

Raw bytes interface fragment (use a separate active node and drive its callback loop; do not reuse the node already shut down above):

```python
node = robot_bus.Node("raw-pilot")
imu = Imu(linear_acceleration=Vector3(z=9.8))
imu_pub = node.create_publisher("/robot1/imu")  # → TopicPublisher
imu_pub.publish(imu.SerializeToString())

def on_raw(payload: bytes):
    imu = Imu()
    imu.ParseFromString(payload)

node.create_subscription("/robot1/imu", on_raw)
```

### WebSocket RPC mode Node (client)

`Node.ws` / `Node.ws_at` (or `Node(..., transport="ws", ws_url=...)`) connect via the broker WebSocket RPC server and do not create ZMQ sockets.

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
# or robot_bus.Node.ws_at("web-client", "http://127.0.0.1:15560")

pub = node.create_publisher("/robot1/cmd")
pub.publish(b"go")

def on_imu(payload: bytes):
    print(len(payload))

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

Use an Executor explicitly when sharing multiple nodes or needing multi-threaded callbacks:

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

Python typed Action servers take one `goal` argument and return `[("FEEDBACK", message), ("RESULT", message)]`; feedback is emitted after the handler returns. For feedback or cancellation checks during execution, use the raw bytes `streaming=True` API: `handler(payload, context) -> bytes`, without typed message arguments.

Run the following programs separately with a broker already running. Start the server callback loop before waiting for results on the main thread; putting `spin()` after a blocking call cannot drive the server.

<!-- runnable: service -->
```python
import os
import robot_bus
from robot_bus.std_srvs.srv.v1 import SetBoolRequest, SetBoolResponse

api_url = os.environ.get("ROBOT_BUS_API_URL", "http://127.0.0.1:15560")
server = robot_bus.Node.discover("worker", transport="tcp", api_url=api_url)
client = robot_bus.Node.discover("caller", transport="tcp", api_url=api_url)

def on_set_bool(req: SetBoolRequest) -> SetBoolResponse:
    return SetBoolResponse(success=True, message=f"set:{req.data}")

try:
    server.create_service(
        "/set_bool", on_set_bool,
        request_type=SetBoolRequest, response_type=SetBoolResponse,
    )
    server.start()
    svc = client.create_client(
        "/set_bool", request_type=SetBoolRequest, response_type=SetBoolResponse,
    )
    if not svc.wait_for_service(timeout=5.0):
        raise TimeoutError("Service /set_bool is not ready")
    reply = svc.call(SetBoolRequest(data=True), timeout=5.0)
    assert reply.success and reply.message == "set:True"
    print(f"service: {reply.success}, {reply.message}")
finally:
    client.shutdown()
    server.shutdown()
    server.stop()
    server.wait()
```

Expected output includes: `service: True, set:True`.

<!-- runnable: action -->
```python
import os
import robot_bus
from robot_bus.example_interfaces.action.v1 import (
    FibonacciGoal, FibonacciFeedback, FibonacciResult,
)

api_url = os.environ.get("ROBOT_BUS_API_URL", "http://127.0.0.1:15560")
server = robot_bus.Node.discover("worker", transport="tcp", api_url=api_url)
client = robot_bus.Node.discover("caller", transport="tcp", api_url=api_url)

def on_fibonacci(goal: FibonacciGoal):
    seq = []
    for i in range(max(0, goal.order)):
        seq.append(i if i < 2 else seq[-1] + seq[-2])
    return [
        ("FEEDBACK", FibonacciFeedback(sequence=seq[:-1])),
        ("RESULT", FibonacciResult(sequence=seq)),
    ]

try:
    server.create_action_server(
        "/fibonacci", on_fibonacci,
        goal_type=FibonacciGoal,
        feedback_type=FibonacciFeedback,
        result_type=FibonacciResult,
    )
    server.start()
    act = client.create_action_client(
        "/fibonacci", goal_type=FibonacciGoal,
        feedback_type=FibonacciFeedback, result_type=FibonacciResult,
    )
    if not act.wait_for_action_server(timeout=5.0):
        raise TimeoutError("Action /fibonacci is not ready")
    goal = act.send_goal(
        FibonacciGoal(order=5),
        feedback_callback=lambda fb: print(f"feedback: {list(fb.sequence)}"),
    )
    result = goal.result(timeout=10.0)
    assert list(result.sequence) == [0, 1, 1, 2, 3]
    print(f"result: {list(result.sequence)}")
finally:
    client.shutdown()
    server.shutdown()
    server.stop()
    server.wait()
```

Expected output includes: `result: [0, 1, 1, 2, 3]`.


Full runnable programs: [`examples/service_set_bool/`](../../examples/service_set_bool/), [`examples/action_fibonacci/`](../../examples/action_fibonacci/).

Raw action: `feedback_callback(body: bytes)`, `ActionGoalHandle.result(timeout=None) -> bytes`; the handle also exposes read-only `goal_id` / `action_name` and `cancel()`. `ActionClient.send_goal_and_wait(...)` keeps the older batch message list usage.
Raw service: `handler(body: bytes) -> bytes` / `call(bytes)`.
The endpoint defaults to the local broker; override with `Node(..., service_frontend=..., service_backend=..., action_backend=..., action_frontend=...)`.

WebSocket and ZMQ both send an explicit `CANCEL` frame. Neither guarantees the server acknowledged cancellation.

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
node.create_subscription("/robot1/imu", lambda p: print(len(p)))

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
    Ros2Bridge,
    StdMsgsStringMapper,
    TopicQos,
    TriggerServiceMapper,
)

assert robot_bus.ros2_available()

bridge = (
    Ros2Bridge.new("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", TopicQos.default())
    .to_bus("/chatter", TopicQos.bus())
    .mapper(StdMsgsStringMapper())
    .add()
    .service()
    .from_ros("/reset", TopicQos.default())
    .to_bus("/reset", TopicQos.bus())
    .mapper(TriggerServiceMapper())
    .add()
    .build()
)
bridge.spin()
```

Key points:

- Configuration is code-only via `.mapper(concrete object)`; no YAML, no type name strings on routes
- Built-in topic mappers: Humble/Jazzy core catalog (~125 types, e.g. `StdMsgsStringMapper`, `SensorMsgsImageMapper`, `GeometryMsgsPoseStampedMapper`); import from `robot_bus.ros2_bridge`. Service/action builtins remain hand-written: `TriggerServiceMapper`, `SetBoolServiceMapper`, `FibonacciActionMapper`
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
| `Node.tcp` / `Node.ipc` / `Node.inproc` / `Node.inproc_with_context` / `Node.with_context` / `Node.ws` / `Node.ws_at` / `Node.discover` | Transport presets (prefer `Context` + `with_context`; WebSocket Nodes operate in client mode; same-process inproc uses `inproc_with_context`; `discover` only fills addresses) |
| `Node.declare_parameter` / `get_parameter` / `set_parameter` / `has_parameter` / `list_parameters` | Local node parameters (`bool` / `int` / `float` / `str`) |
| `Node.load_parameters_from_yaml_str` / `load_parameters_from_yaml_file` | Load / override parameters from YAML |
| `node.spin()` / `spin_once` / `shutdown` | Drive callbacks (ROS 2–style simple path) |
| `node.connection_state` / `wait_for_broker(timeout=None)` / `add_on_connection_event` | Broker link: construct does not block; wait or observe `connected` / `reconnecting` |
| `node.wait_for_message(topic, timeout=None)` | Wait for one message or timeout (`bytes` / `None`) |
| `Context()` | Shared ZMQ context (required for same-process inproc) |
| `SingleThreadedExecutor(context=None)` | Explicit single-threaded executor (for shared multi-node use) |
| `MultiThreadedExecutor(num_threads=4, context=None)` | n resident workers; subscriptions / timers / services / actions follow callback groups |
| `executor.add_node(node)` | Attach node to executor (must be before auto-attach on that node) |
| `node.create_publisher(topic, msg_type=None, qos_depth=None)` | typed → `TypedTopicPublisher`; omit type → raw; `qos_depth>0` → KeepLast HWM (ignored on WS publish) |
| `node.create_timer(period, callback)` → `TimerHandle` | Timer (attached to Node like topic) |
| `CallbackGroupType` / `create_callback_group` | `MutuallyExclusive` / `Reentrant` |
| `create_subscription(..., msg_type=, callback_group=, qos_depth=)` | typed: `callback(Message)`; omit type: `callback(bytes)`; WS: `qos_depth` sizes the server subscribe queue |
| `create_service(..., request_type=, response_type=, qos_depth=)` | typed: `handler(Request) -> Response`; otherwise raw bytes; `qos_depth>0` → KeepLast DEALER HWM |
| `create_client(..., request_type=, response_type=, qos_depth=)` | typed → `TypedServiceClient`; `service_is_ready` / `wait_for_service` (console workers); `qos_depth>0` → KeepLast DEALER HWM |
| `create_action_server(..., goal_type=, feedback_type=, result_type=, qos_depth=)` | typed `handler(goal)` returns phase/message pairs, emitted after return; live raw handlers use `streaming=True` without typed arguments; `qos_depth>0` → KeepLast DEALER HWM |
| `create_action_client(..., goal_type=, feedback_type=, result_type=, qos_depth=)` | typed → `TypedActionClient`; `wait_for_action_server`; `send_goal` → GoalHandle; `qos_depth>0` → KeepLast DEALER HWM |
| `ActionGoalHandle` / `TypedActionGoalHandle` | Goal id, action name, blocking wait for result, best-effort cancel |
| `Publisher(endpoint=None)` | Low-level XSUB connection (without Node) |
| `ros2_available()` | Whether `import rclpy` succeeds (native Python bridge) |
| `robot_bus.ros2_bridge.Ros2Bridge` / `Direction` / built-in Mapper | In-process ROS bridge (**rclpy**); see [ros2-bridge.md](ros2-bridge.md) |
| `RobotBusBroker.start(...)` / `python -m robot_bus.broker` | In-process `start`; standalone `python -m robot_bus.broker` (same CLI flags); pass `context` for same-process inproc; peers use CLI-style string lists |
| `ShutdownHandle` / `TimerHandle` | Spin and timer control |

WebSocket RPC mode Node: see previous section; for the low-level WebSocket RPC frame protocol, see the Rust guide ([rust-api.md](rust-api.md)).
