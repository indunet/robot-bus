[English](../en/api-compare.md) | 中文

# API 对比：ROS 2 Humble（rclrs）↔ robot-bus

几个经典场景下，两侧怎么写。左侧是 **ROS 2 Humble + [rclrs](https://github.com/ros2-rust/ros2_rust)**，右侧是 **robot-bus**。

| 概念 | ROS 2 Humble（rclrs） | robot-bus |
|------|------------------------|-----------|
| 运行时 | DDS（需 `ros2` / daemon） | 先起 `robot_bus_broker`（或进程内嵌入） |
| 入口 | `Context` → `Node` → `rclrs::spin` | **推荐** `Context` → `Node::with_context`；tcp/ipc 仍可用便捷的 `Node::new`（私有 Context） |
| 消息 | `.msg` / `.srv` / `.action` 生成类型 | crate 内 protobuf（如 `sensor_msgs::msg::v1::Imu`） |
| QoS | `QOS_PROFILE_DEFAULT` 等 | Topic：`QosProfile::keep_last(depth)` → ZMQ 上为 HWM（各语言可选 `qos_depth` / `qosDepth`）；固定 best-effort。WebSocket 订阅用同一 depth 作为网关到客户端的队列；WS 发布忽略 QoS（共用网关 PUB）。Service / action 暂不接 QoS |
| 回调组 | Worker / callback group（较新 API） | `CallbackGroupType::MutuallyExclusive` / `Reentrant` |
| 参数 | `declare_parameter` / `get_parameter` → Parameter；`set_parameter(Parameter)`；`list_parameters(prefixes, depth)`（可远程 / YAML / CLI） | 同形本地 API（`Parameter` + `as_*` + 批量 get/set）；`list_parameters` → `{names, prefixes}`，便利 API `list_all_parameters`；`undeclare_parameter`；YAML 加载；无远程 / CLI |
| 就绪等待 | `wait_for_message` / `wait_for_service` / `wait_for_action_server` | 同名辅助：`wait_for_message`；service/action 通过 console `workers > 0` 轮询（best-effort，非 DDS discovery）。另有与 broker 的会话：`connection_state` / `wait_for_broker`（构造不阻塞；TCP/WS 自动重连） |
| 销毁 | `destroy_subscription` / destroy service·action server | 同形：按 handle id 销毁；`start()` 活跃时与 `cancel_timer` 一样拒绝 |
| 定时器 | `create_wall_timer` | `create_timer` / 别名 `create_wall_timer` |

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
    let context = Context::new();
    let mut node = Node::with_context(&context, "pilot");
    // create_* ...
    node.spin()?; // 内部用 SingleThreadedExecutor
    Ok(())
}
```

便捷写法（私有 Context，适合单节点 tcp/ipc）：`Node::new("pilot")`。

同进程 **inproc** 时必须与嵌入式 broker 共享同一 Context：

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
        None, // callback group，None = 默认互斥组
    )?;

    publisher.publish(&Imu::default())?;
    node.spin()?;
    Ok(())
}
```

要点：rclrs 创建时要带完整 QoS；robot-bus 的 `QosProfile` **仅对 topic 生效**，且只兑现 KeepLast depth（ZMQ 上 → 发送/接收 HWM；WebSocket 上 → 网关订阅队列）。reliability 固定 best-effort。WS **发布** QoS 忽略（共用网关 PUB）。不传 QoS 的 `create_publisher` / `create_subscription` 仍可用（不改动已有 HWM）。第三个参数是 callback group。topic 名按传入原样使用（建议写全路径）。

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

rclrs 0.7 provides `create_action_server` / `create_action_client`. robot-bus uses the same ROS 2–style split between an immediately returned GoalHandle, streaming feedback, and a separately awaited result. The optional ROS 2 bridge is implemented natively per language (rclrs / rclpy / rclcpp) for topic, service, and action with concrete mappers (no YAML / no type-string mounting).

**rclrs**

```rust
// node.create_action_server / create_action_client
// send_goal 立即返回 goal handle
// feedback callback 实时接收；result future/handle 独立等待；handle 可请求 cancel
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
    )?; // GoalHandle 立即返回

    // goal.cancel()?; // best-effort，不表示服务端已确认
    let result = goal.result(Some(Duration::from_secs(10)))?;

    Ok(())
}
```

各传输上 `cancel` 均为 best-effort：浏览器 WebSocket 发显式 `CANCEL` 帧并继续等到 RESULT（真断连仍 cancel）；原生 WebSocket RPC 同样发 CANCEL；ZMQ 发显式 `CANCEL` 帧。均不承诺服务端确认。

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
| 建节点 | `Node::new(&context, "name")` | `Node::with_context(&context, "name")`（或便捷 `Node::new("name")`） |
| 发布 | `create_publisher::<T>(topic, qos)` | `create_publisher_with_qos::<T>(topic, qos)`（或无 QoS 的 `create_publisher`；各语言可选 depth） |
| 订阅 | `create_subscription(topic, qos, cb)` | `create_subscription_with_qos(topic, qos, cb, group)`（或无 QoS 的 `create_subscription`） |
| 服务端 | `create_service::<S, _>(name, cb)` | `create_service::<S, _>(name, cb, group)` |
| 客户端 | `create_client::<S>(name)` + `call` | `create_client::<S>(name)` + `call(..., timeout)` + `wait_for_service` |
| Action 服务端 | `create_action_server` | `create_action_server::<A, _>(..., group)` |
| Action 客户端 | `create_action_client` + GoalHandle | `create_action_client` + `wait_for_action_server` + `send_goal` → GoalHandle |
| 转起来 | `rclrs::spin(node)` | `node.spin()` / `wait_for_message` |
| 原始字节 | 动态消息 / 有限支持 | `create_*_raw` |

更完整的 robot-bus 示例见 [rust-api.md](./rust-api.md)。
