English | [中文](../zh/api-compare.md)

# API compare: ROS 2 Humble (rclrs) ↔ robot-bus

How to write the same classic scenarios on each side. Left is **ROS 2 Humble + [rclrs](https://github.com/ros2-rust/ros2_rust)**, right is **robot-bus**.

| Concept | ROS 2 Humble (rclrs) | robot-bus |
|---------|------------------------|-----------|
| Runtime | DDS (requires `ros2` / daemon) | Start `robot_bus_broker` first (or embed in-process) |
| Entry | `Context` → `Node` → `rclrs::spin` | **Preferred** `Context` → `Node::with_context`; tcp/ipc may still use convenience `Node::new` (private Context) |
| Messages | `.msg` / `.srv` / `.action` generated types | Protobuf in crate (e.g. `sensor_msgs::msg::v1::Imu`) |
| QoS | `QOS_PROFILE_DEFAULT`, etc. | Topic: `QosProfile::keep_last(depth)` → HWM (optional `qos_depth` / `qosDepth` in all bindings); fixed best-effort. WebSocket RPC Node (`Node::ws`) accepts the arg but ignores it. Service / action do not take QoS yet |
| Callback groups | Worker / callback group (newer API) | `CallbackGroupType::MutuallyExclusive` / `Reentrant` |
| Parameters | `declare_parameter` / `get_parameter` → Parameter; `set_parameter(Parameter)`; `list_parameters(prefixes, depth)` (remote / YAML / CLI) | Same local shape (`Parameter` + `as_*` + batch get/set); `list_parameters` → `{names, prefixes}`, convenience `list_all_parameters`; `undeclare_parameter`; YAML load; no remote / CLI |
| Ready waits | `wait_for_message` / `wait_for_service` / `wait_for_action_server` | Same helpers: `wait_for_message`; service/action poll console `workers > 0` (best-effort, not DDS discovery) |
| Destroy | `destroy_subscription` / destroy service·action server | Same: destroy by handle id; rejected while `start()` is active (like `cancel_timer`) |
| Timers | `create_wall_timer` | `create_timer` / alias `create_wall_timer` |

---

## 1. Node + spin

**rclrs (Humble)**

```rust
use rclrs::{Context, Node};

fn main() -> Result<(), rclrs::RclrsError> {
    let context = Context::new(std::env::args())?;
    let node = Node::new(&context, "pilot")?;
    // create_* ...
    rclrs::spin(node)
}
```

**robot-bus**

```rust
use robot_bus::{Context, Node};

fn main() -> robot_bus::Result<()> {
    let context = Context::new();
    let mut node = Node::with_context(&context, "pilot");
    // create_* ...
    node.spin()?; // uses SingleThreadedExecutor internally
    Ok(())
}
```

Convenience (private Context, fine for single-node tcp/ipc): `Node::new("pilot")`.

Same-process **inproc** requires a shared Context:

```rust
use robot_bus::{Context, Node, RobotBusBroker, RobotBusConfig};

fn main() -> anyhow::Result<()> {
    let ctx = Context::new();
    let broker = RobotBusBroker::start_with_context(&ctx, RobotBusConfig::default())?;
    let mut node = Node::inproc_with_context(&ctx, "pilot");
    // create_* ...
    node.spin()?;
    broker.stop()?;
    Ok(())
}
```

---

## 2. Pub / Sub

**rclrs (Humble)**

```rust
use rclrs::{Context, Node, QOS_PROFILE_DEFAULT};
use std_msgs::msg::String as StringMsg;

fn main() -> Result<(), rclrs::RclrsError> {
    let context = Context::new(std::env::args())?;
    let node = Node::new(&context, "talker")?;

    let publisher = node.create_publisher::<StringMsg>("chatter", QOS_PROFILE_DEFAULT)?;
    let _sub = node.create_subscription::<StringMsg, _>(
        "chatter",
        QOS_PROFILE_DEFAULT,
        |msg: StringMsg| {
            println!("heard: {}", msg.data);
        },
    )?;

    let mut msg = StringMsg { data: "hello".into() };
    publisher.publish(&msg)?;

    rclrs::spin(node)
}
```

**robot-bus**

```rust
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::{Context, Node, QosProfile};

fn main() -> robot_bus::Result<()> {
    let ctx = Context::new();
    let mut node = Node::with_context(&ctx, "talker");

    let publisher = node.create_publisher_with_qos::<Imu>(
        "/robot1/imu",
        QosProfile::keep_last(10),
    )?;
    node.create_subscription_with_qos::<Imu, _>(
        "/robot1/imu",
        QosProfile::keep_last(10),
        |_topic, imu| {
            println!("angular_z={:?}", imu.angular_velocity);
        },
        None, // callback group; None = default mutually exclusive group
    )?;

    publisher.publish(&Imu::default())?;
    node.spin()?;
    Ok(())
}
```

Key points: rclrs requires full QoS at creation; robot-bus `QosProfile` applies to **topics only**, and only KeepLast depth is honored (→ send/receive HWM). Reliability is fixed best-effort. Plain `create_publisher` / `create_subscription` still work (leave existing HWM alone). The last argument is the callback group. robot-bus uses topic names as given (prefer fully qualified paths).

---

## 3. Service

**rclrs (Humble)**

```rust
use example_interfaces::srv::{AddTwoInts, AddTwoInts_Request, AddTwoInts_Response};
use rclrs::{Context, Node};

fn main() -> Result<(), rclrs::RclrsError> {
    let context = Context::new(std::env::args())?;
    let node = Node::new(&context, "add_server")?;

    let _svc = node.create_service::<AddTwoInts, _>(
        "add_two_ints",
        |req: AddTwoInts_Request| AddTwoInts_Response {
            sum: req.a + req.b,
        },
    )?;

    // Client: node.create_client::<AddTwoInts>("add_two_ints")?
    //         .call(req)?  (exact API varies slightly by rclrs minor version)

    rclrs::spin(node)
}
```

**robot-bus**

```rust
use std::time::Duration;
use robot_bus::std_srvs::srv::v1::{SetBool, SetBoolRequest, SetBoolResponse};
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut server = Node::new("svc_server");
    let mut client_node = Node::new("svc_client");

    server.create_service::<SetBool, _>(
        "/set_bool",
        |req: SetBoolRequest| SetBoolResponse {
            success: true,
            message: format!("set:{}", req.data),
        },
        None,
    )?;

    let client = client_node.create_client::<SetBool>("/set_bool")?;
    let resp = client.call(&SetBoolRequest { data: true }, Some(Duration::from_secs(5)))?;
    assert!(resp.success);

    server.spin()?;
    Ok(())
}
```

---

## 4. Action

rclrs 0.7 provides `create_action_server` / `create_action_client`. robot-bus uses the same ROS 2–style split between an immediately returned GoalHandle, streaming feedback, and a separately awaited result. The optional ROS 2 bridge is implemented natively per language (rclrs / rclpy / rclcpp) for topic, service, and action with concrete mappers (no YAML / no type-string mounting).

**rclrs**

```rust
// node.create_action_server / create_action_client
// send_goal returns goal handle immediately
// feedback callback receives updates in real time; result future/handle awaited separately; handle can request cancel
```

**robot-bus**

```rust
use std::time::Duration;
use robot_bus::example_interfaces::action::v1::{Fibonacci, FibonacciGoal};
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut client_node = Node::new("act_client");
    let client = client_node.create_action_client::<Fibonacci>("fibonacci")?;
    let goal = client.send_goal(
        &FibonacciGoal { order: 5 },
        |feedback| println!("feedback: {:?}", feedback.sequence),
    )?; // GoalHandle returned immediately

    // goal.cancel()?; // best-effort; does not mean server confirmed
    let result = goal.result(Some(Duration::from_secs(10)))?;

    Ok(())
}
```

Cancel is best-effort on every transport: browser WebSocket sends an explicit `CANCEL` frame and waits until RESULT (true disconnect still cancels); native WebSocket RPC does the same; ZMQ sends an explicit `CANCEL` frame. None guarantee server acknowledgment.

---

## 5. Timer

**rclrs (Humble)**

```rust
use std::time::Duration;
// Common pattern: spawn thread with sleep + publish, or newer API create_timer
std::thread::spawn(move || {
    loop {
        std::thread::sleep(Duration::from_millis(100));
        // publisher.publish(...)
    }
});
rclrs::spin(node)?;
```

**robot-bus**

```rust
use std::sync::Arc;
use std::time::Duration;

node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| {
        // periodic task
    }),
    None,
)?;
node.spin()?;
```

---

## Quick reference

| Scenario | rclrs | robot-bus |
|----------|-------|-----------|
| Create node | `Node::new(&context, "name")` | `Node::with_context(&context, "name")` (or convenience `Node::new("name")`) |
| Publish | `create_publisher::<T>(topic, qos)` | `create_publisher_with_qos::<T>(topic, qos)` (or plain `create_publisher`; bindings expose optional depth) |
| Subscribe | `create_subscription(topic, qos, cb)` | `create_subscription_with_qos(topic, qos, cb, group)` (or plain `create_subscription`) |
| Service server | `create_service::<S, _>(name, cb)` | `create_service::<S, _>(name, cb, group)` |
| Service client | `create_client::<S>(name)` + `call` | `create_client::<S>(name)` + `call(..., timeout)` + `wait_for_service` |
| Action server | `create_action_server` | `create_action_server::<A, _>(..., group)` |
| Action client | `create_action_client` + GoalHandle | `create_action_client` + `wait_for_action_server` + `send_goal` → GoalHandle |
| Spin | `rclrs::spin(node)` | `node.spin()` / `wait_for_message` |
| Raw bytes | Dynamic messages / limited support | `create_*_raw` |

For more complete robot-bus examples, see [rust-api.md](./rust-api.md).
