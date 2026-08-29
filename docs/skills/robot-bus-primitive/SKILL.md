---
name: robot-bus-primitive
description: >-
  Create robot-bus Node publishers, subscriptions, services, action servers/clients,
  and timers (typed protobuf or raw bytes). Use when the user asks to write a bus
  node, add pub/sub, RPC service, action goal/feedback/result, create_timer, spin,
  QoS keep_last, callback groups, or broker-backed messaging without ROS migration.
---

# robot-bus primitive (pub / sub / service / action / timer)

Guide for implementing graph entities on **robot-bus** only. Prefer project docs
over inventing signatures:
[api-compare.md](../zh/api-compare.md), [rust-api.md](../zh/rust-api.md),
[python-api.md](../zh/python-api.md), [cpp-api.md](../zh/cpp-api.md).

For ROS ↔ bus interconnect, use skill **ros2-bridge** / [ros2-bridge.md](../zh/ros2-bridge.md).
For full ROS package migration, use **ros2-to-robot-bus** or **robot-bus-to-ros2**.

## Prerequisites

1. Start a broker (or embed `RobotBusBroker` in-process).
2. Create a `Node`, attach create_* APIs, then `spin` / `spin_once`.

```bash
robot-bus-broker
# or: cargo run --bin robot_bus_broker
```

Default console / discover API: `http://127.0.0.1:15570`.

| Transport | Notes |
|-----------|--------|
| tcp (default `Node::new` / `Node("name")`) | Cross-process; discover fills addresses |
| ipc | Same machine |
| inproc | **Must** share one `Context` with embedded broker |
| ws gateway | Client: pub/sub + call service/action; **cannot** be service/action server |

Prefer **typed** APIs (protobuf bound at create). Use `*_raw` only for opaque payloads.

## Workflow checklist

```
Progress:
- [ ] 1. Broker up (external or inproc + shared Context)
- [ ] 2. Node created (name + transport)
- [ ] 3. Messages: built-in *.msg.v1 / *.srv.v1 / action.v1 or project proto
- [ ] 4. create_publisher / create_subscription (+ optional QoS)
- [ ] 5. create_service / create_client and/or create_action_* as needed
- [ ] 6. create_timer if periodic work
- [ ] 7. Callback groups / MultiThreadedExecutor only if needed
- [ ] 8. spin; verify in console or smoke publish/call
```

## API cheat sheet

| Need | robot-bus |
|------|-----------|
| Node | `Node::new("name")` / `Node.with_context` / `Node("name")` |
| Spin | `node.spin()` (auto SingleThreadedExecutor) |
| Publish | `create_publisher` / `create_publisher_with_qos` |
| Subscribe | `create_subscription` / `_with_qos` — cb is `msg` only |
| Service server | `create_service` / `create_service_with_qos` |
| Service client | `create_client` / `create_client_with_qos` → `call(req, timeout)` |
| Action server | `create_action_server` / `create_action_server_with_qos` |
| Action client | `create_action_client` / `create_action_client_with_qos` → `send_goal` → GoalHandle (`result` / `cancel`) |
| Timer | `create_timer(period, cb, group?)` |
| QoS | `QosProfile::keep_last(depth)` → ZMQ HWM (topic PUB/SUB; service / action DEALER); WS subscribe → gateway queue; WS publish ignored; reliability **best-effort** |
| Groups | `MutuallyExclusive` (default) / `Reentrant` (+ MultiThreadedExecutor) |

Topic / service / action **names are used as given** — prefer absolute paths like `/robot1/imu`.

## 1. Pub / Sub

### Rust

```rust
use robot_bus::{Node, QosProfile};
use robot_bus::sensor_msgs::msg::v1::Imu;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("talker");
    let pub_ = node.create_publisher_with_qos::<Imu>(
        "/robot1/imu",
        QosProfile::keep_last(10),
    )?;
    node.create_subscription_with_qos::<Imu, _>(
        "/robot1/imu",
        QosProfile::keep_last(10),
        |imu| { let _ = imu; },
        None, // default mutually exclusive group
    )?;
    pub_.publish(&Imu::default())?;
    node.spin()?;
    Ok(())
}
```

Raw: `create_publisher_raw` / `create_subscription_raw`.

### Python

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu

node = robot_bus.Node("talker")
pub = node.create_publisher("/robot1/imu", Imu)
node.create_subscription("/robot1/imu", lambda msg: None, msg_type=Imu)
pub.publish(Imu())
# node.spin()
```

Omit `msg_type` / second type arg for raw `bytes`.

## 2. Service

### Rust

```rust
use std::time::Duration;
use robot_bus::std_srvs::srv::v1::{SetBool, SetBoolRequest, SetBoolResponse};
use robot_bus::Node;

let mut server = Node::new("svc_server");
server.create_service::<SetBool, _>(
    "/set_bool",
    |req: SetBoolRequest| SetBoolResponse {
        success: true,
        message: format!("set:{}", req.data),
    },
    None,
)?;

let mut client_node = Node::new("svc_client");
let client = client_node.create_client::<SetBool>("/set_bool")?;
let resp = client.call(&SetBoolRequest { data: true }, Some(Duration::from_secs(5)))?;
```

Server must `spin` for handlers to run. Raw: `create_service_raw` / `create_client_raw`.

### Python

```python
from robot_bus.std_srvs.srv.v1 import SetBoolRequest, SetBoolResponse

def on_set_bool(req: SetBoolRequest) -> SetBoolResponse:
    return SetBoolResponse(success=True, message=f"set:{req.data}")

server = robot_bus.Node("worker")
server.create_service(
    "/set_bool", on_set_bool,
    request_type=SetBoolRequest, response_type=SetBoolResponse,
)
cli = robot_bus.Node("caller").create_client(
    "/set_bool",
    request_type=SetBoolRequest, response_type=SetBoolResponse,
)
# reply = cli.call(SetBoolRequest(data=True), timeout=5.0)
```

## 3. Action

ROS 2–style GoalHandle: `send_goal` returns immediately; feedback via callback; `result(timeout)` waited separately; `cancel()` is best-effort (no server ack guarantee).

### Rust (client sketch)

```rust
use std::time::Duration;
use robot_bus::example_interfaces::action::v1::{Fibonacci, FibonacciGoal};
use robot_bus::Node;

let mut node = Node::new("act_client");
let client = node.create_action_client::<Fibonacci>("fibonacci")?;
let goal = client.send_goal(
    &FibonacciGoal { order: 5 },
    |feedback| println!("{:?}", feedback.sequence),
)?;
// goal.cancel()?;
let result = goal.result(Some(Duration::from_secs(10)))?;
```

Server: `create_action_server::<A, _>(...)`. Confirm exact action type path in rust-api / crate docs (`action::v1` vs `robot_bus_interfaces`).

### Python

```python
from robot_bus.example_interfaces.action.v1 import (
    FibonacciGoal, FibonacciFeedback, FibonacciResult,
)

def on_fibonacci(goal: FibonacciGoal, context):
    seq = list(range(goal.order))
    context.publish_feedback(FibonacciFeedback(sequence=seq[:1]))
    return FibonacciResult(sequence=seq)

server = robot_bus.Node("worker")
server.create_action_server(
    "/fibonacci", on_fibonacci,
    goal_type=FibonacciGoal,
    feedback_type=FibonacciFeedback,
    result_type=FibonacciResult,
)

act = robot_bus.Node("caller").create_action_client(
    "/fibonacci",
    goal_type=FibonacciGoal,
    feedback_type=FibonacciFeedback,
    result_type=FibonacciResult,
)
handle = act.send_goal(
    FibonacciGoal(order=5),
    feedback_callback=lambda fb: print(fb.sequence),
)
result = handle.result(timeout=10.0)
```

## 4. Timer

Callbacks run under `spin` / `spin_once` (same as subscriptions).

### Rust

```rust
use std::sync::Arc;
use std::time::Duration;

node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| { /* tick */ }),
    None,
)?;
```

### Python

```python
handle = node.create_timer(0.1, lambda: print("tick"))  # seconds
# node.cancel_timer(handle)
```

## 5. Spin, groups, executor

- Single node: `node.spin()` is enough.
- Stop from another thread: `shutdown_handle().shutdown()`.
- Non-blocking: `spin_once(timeout)`.
- Default callback group = mutually exclusive. `Reentrant` only parallelizes with `MultiThreadedExecutor`.
- WS Node (`Node::ws`): can publish/subscribe/call; cannot host service/action servers.

## 6. Message types

1. Prefer built-ins: `std_msgs`, `geometry_msgs`, `sensor_msgs`, `std_srvs`, action Fibonacci, etc. (`*.msg.v1` / `*.srv.v1`).
2. Custom interfaces → project `.proto` + language codegen; wire names like `sensor_msgs.msg.v1.Imu`.
3. Do not assume DDS CDR on the bus.

## 7. Verification

1. Broker running; open `http://127.0.0.1:15570` (or `rbus topic list`).
2. Smoke: one publish/subscribe pair, one service call, one action goal if used.
3. Timer: confirm ticks only while spinning.

## Deliverables when implementing for the user

1. Node source with the requested create_* APIs
2. Broker run instructions (CLI or inproc)
3. Typed imports matching existing SDK packages
4. Notes on QoS limits (KeepLast depth only: topic PUB/SUB HWM, service/action DEALER HWM) and cancel semantics if actions are used

Read language docs for C++/Java/TS/Android rather than inventing alternate APIs here.
