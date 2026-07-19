# API 对比：ROS 2 Humble（rclrs）↔ robot-bus

几个经典场景下，两侧怎么写。左侧是 **ROS 2 Humble + [rclrs](https://github.com/ros2-rust/ros2_rust)**，右侧是 **robot-bus**。

| 概念 | ROS 2 Humble（rclrs） | robot-bus |
|------|------------------------|-----------|
| 运行时 | DDS（需 `ros2` / daemon） | 先起 `robot_bus_broker`（或进程内嵌入） |
| 入口 | `Context` → `Node` → `rclrs::spin` | `Context` → `Node` / `RobotBusBroker::start_with_context`（tcp/ipc 仍可用无 Context 的 `Node::new`） |
| 消息 | `.msg` / `.srv` / `.action` 生成类型 | crate 内 protobuf（如 `sensor_msgs::msg::v1::Imu`） |
| QoS | `QOS_PROFILE_DEFAULT` 等 | 无 ROS QoS；可用 HWM |
| 回调组 | Worker / callback group（较新 API） | `CallbackGroupType::MutuallyExclusive` / `Reentrant` |

---

## 1. Node + spin

**rclrs（Humble）**

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
    // tcp/ipc: Node::new is fine (private context).
    let mut node = Node::new("pilot");
    // create_* ...
    node.spin()?; // 内部用 SingleThreadedExecutor
    Ok(())
}
```

同进程 **inproc** 时必须共享 Context：

```rust
use robot_bus::{Context, Node, RobotBusBroker, RobotBusConfig};

fn main() -> anyhow::Result<()> {
    let ctx = Context::new();
    let broker = RobotBusBroker::start_with_context(ctx.clone(), RobotBusConfig::default())?;
    let mut node = Node::inproc_with_context(ctx, "pilot");
    // create_* ...
    node.spin()?;
    broker.stop()?;
    Ok(())
}
```

---

## 2. Pub / Sub

**rclrs（Humble）**

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
use robot_bus::Node;

fn main() -> robot_bus::Result<()> {
    let mut node = Node::new("talker");

    let publisher = node.create_publisher::<Imu>("/robot1/imu")?;
    node.create_subscription::<Imu, _>(
        "/robot1/imu",
        |_topic, imu| {
            println!("angular_z={:?}", imu.angular_velocity);
        },
        None, // callback group，None = 默认互斥组
    )?;

    publisher.publish(&Imu::default())?;
    node.spin()?;
    Ok(())
}
```

要点：rclrs 创建时要带 QoS；robot-bus 第三个参数是 callback group。topic 名 robot-bus 按传入原样使用（建议写全路径）。

---

## 3. Service

**rclrs（Humble）**

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

    // 客户端：node.create_client::<AddTwoInts>("add_two_ints")?
    //         .call(req)?  （具体 API 随 rclrs 小版本略有差异）

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

Humble 时期 rclrs 的 action 支持不完整；下面左侧用较新 rclrs 风格示意概念，右侧为 robot-bus 现写法。

**rclrs（概念）**

```rust
// node.create_action_server / create_action_client
// goal → feedback 流 → result；需 executor.spin
```

**robot-bus**

```rust
use std::time::Duration;
use robot_bus::action::v1::{Fibonacci, FibonacciFeedback, FibonacciGoal, FibonacciResult};
use robot_bus::{ActionOutcome, Node};

fn main() -> robot_bus::Result<()> {
    let mut server = Node::new("act_server");
    let mut client_node = Node::new("act_client");

    server.create_action_server::<Fibonacci, _>(
        "fibonacci",
        |goal: FibonacciGoal| {
            let order = goal.order.max(0) as usize;
            let mut seq = vec![0i32, 1];
            while seq.len() < order {
                let n = seq[seq.len() - 1] + seq[seq.len() - 2];
                seq.push(n);
            }
            seq.truncate(order);
            ActionOutcome {
                feedbacks: vec![FibonacciFeedback {
                    sequence: seq.clone(),
                }],
                result: FibonacciResult { sequence: seq },
            }
        },
        None,
    )?;

    let client = client_node.create_action_client::<Fibonacci>("fibonacci")?;
    let outcome = client.send_goal(
        &FibonacciGoal { order: 5 },
        None,
        Some(Duration::from_secs(10)),
    )?;

    server.spin()?;
    Ok(())
}
```

---

## 5. Timer

**rclrs（Humble）**

```rust
use std::time::Duration;
// 常见写法：另起线程 sleep + publish，或较新 API 的 create_timer
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
        // 周期任务
    }),
    None,
)?;
node.spin()?;
```

---

## 对照速查

| 场景 | rclrs | robot-bus |
|------|-------|-----------|
| 建节点 | `Node::new(&context, "name")` | `Node::new("name")` |
| 发布 | `create_publisher::<T>(topic, qos)` | `create_publisher::<T>(topic)` |
| 订阅 | `create_subscription(topic, qos, cb)` | `create_subscription(topic, cb, group)` |
| 服务端 | `create_service::<S, _>(name, cb)` | `create_service::<S, _>(name, cb, group)` |
| 客户端 | `create_client::<S>(name)` + `call` | `create_client::<S>(name)` + `call(..., timeout)` |
| Action 服务端 | `create_action_server`（Humble 弱） | `create_action_server::<A, _>(..., group)` |
| Action 客户端 | `create_action_client` | `create_action_client` + `send_goal` |
| 转起来 | `rclrs::spin(node)` | `node.spin()` |
| 原始字节 | 动态消息 / 有限支持 | `create_*_raw` |

更完整的 robot-bus 示例见 [rust-api.md](./rust-api.md)。
