[English](../en/typescript-api.md) | 中文

# TypeScript API

```bash
npm install robot-bus
# 本地：just ts-dev
# 等价：cd bindings/typescript && npm install && npm run build:native && npm run build:ts
```

单一 npm 包，按运行环境自动选入口（`package.json` `exports`）：

| 环境 | 入口 | 能力 |
|------|------|------|
| Node.js | napi-rs 原生扩展 | 完整 ZMQ Node（publish、service/action server、本地 broker） |
| 浏览器 | WebSocket RPC（`/ws`） | 订阅 / publish / 调 service / action（无 server） |

`console/` Web UI **不是**本 SDK。

## Broker

与 Rust / Python 相同：先起 broker，再跑业务代码。

```bash
# Rust / Python CLI
robot-bus-broker --grpc-listen 0.0.0.0:15570 --tcp-only
```

Node 进程内：

```ts
import { RobotBusBroker } from "robot-bus";

const broker = RobotBusBroker.start({
  grpcListen: "0.0.0.0:15570",
  tcpOnly: true,
  noConsole: true,
});
// …业务…
broker.stop();
```

### UDP discovery（仅 Node.js 原生）

```ts
import { Node, RobotBusBroker } from "robot-bus";

const broker = RobotBusBroker.start({ domainId: 0, advertiseHost: "127.0.0.1" });
const node = Node.discover("talker", { transport: "tcp", domainId: 0 });
```

浏览器入口无 UDP 发现，请用显式 `wsUrl`（HTTP 原点；SDK 会连 `ws://…/ws`）。

跨 broker（federation）与 CLI 同款字符串约定：

```ts
const broker = RobotBusBroker.start({
  brokerId: "broker-a",
  messagePeers: ["tcp://10.0.0.2:15561"],
  servicePeers: ["broker-b=tcp://10.0.0.2:15663"],
  actionPeers: ["broker-b=tcp://10.0.0.2:15665"],
  tcpOnly: true,
  noConsole: true,
});
broker.stop();
```

同进程 **inproc** 须共享 `Context`：

```ts
import { Context, Node, RobotBusBroker } from "robot-bus";

const ctx = new Context();
const broker = RobotBusBroker.start({ noConsole: true }, ctx);
const node = Node.inprocWithContext(ctx, "pilot");
```

tcp / ipc / ws 不要求共享 Context。

## 本地参数（Node.js）

```ts
const node = new Node("pilot");
node.declareParameter("max_speed", 1.5);
node.declareParameter("frame_id", "base_link");
node.setParameter("max_speed", 2.0);
console.log(node.getParameter("max_speed"));
node.loadParametersFromYamlStr("ros__parameters:\n  max_speed: 3.0\n");
// node.loadParametersFromYamlFile("config/pilot.yaml");
```

## Node.js（完整 API）

```ts
import { Node, RobotBusBroker } from "robot-bus";
import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";

const broker = RobotBusBroker.start({ tcpOnly: true, noConsole: true });
const node = new Node("pilot");

const pub = node.createPublisher("/robot1/imu", Imu);
node.createSubscription(
  "/robot1/imu",
  (_topic, imu) => {
    console.log(imu);
  },
  Imu,
);

pub.publish(
  Imu.create({
    /* … */
  }),
);

// node.spin();
broker.stop();
```

WebSocket RPC 客户端模式（不启 ZMQ）：

```ts
const node = Node.ws("web-client");
// 或 Node.wsAt("web-client", "http://127.0.0.1:15570");
```

| 支持 | 不支持（WS 模式） |
|------|---------------------|
| `createSubscription` | `createService` |
| `createPublisher` | `createActionServer` |
| `createClient` | — |
| `createActionClient` | |
| `createTimer`、`spin` / `shutdown` | |

## TF（坐标树，Node.js only）

`TfBuffer` / `TfListener` / `TransformBroadcaster` 对应 Rust `robot_bus::tf`。浏览器入口不提供原生 Buffer。消息为 `tf2_msgs/TFMessage`（`/tf`、`/tf_static`）。

```ts
import {
  createTfBuffer,
  TfListener,
  TransformBroadcaster,
} from "robot-bus";
import { TFMessage } from "robot-bus/tf2_msgs/msg/v1/tf_message.js";
import { TransformStamped } from "robot-bus/geometry_msgs/msg/v1/stamped.js";

const listener = new TfListener(node); // /tf + /tf_static
const buf = listener.buffer();
const br = TransformBroadcaster.fromTyped(
  node.createPublisher("/tf_static", TFMessage),
  TFMessage,
);
br.send(msg);

const t = buf.lookupTransform("base_link", "camera", TransformStamped);
```

离线：`createTfBuffer()` + `setTransformMsg`。见 `tests/tf_lookup.test.ts`。

## 浏览器（WebSocket RPC）

浏览器客户端走 broker 的 **`/ws`**（类 gRPC over WebSocket，**一条连接多路复用（V2 stream_id）**），不再使用 gRPC-Web。

```ts
import { Node } from "robot-bus";
// bundler 自动解析到 browser 入口

const node = Node.ws("browser-client"); // 默认 http://127.0.0.1:15570 → ws://127.0.0.1:15570/ws
const pub = node.createPublisher("/robot1/cmd");
await pub.publish(new TextEncoder().encode("go"));
node.createSubscription("/robot1/imu", (topic, payload) => {
  console.log(topic, payload);
});
node.start(); // 或 node.spin()
```

浏览器下 `createService` / `createActionServer` 会抛错。

也可显式使用：

```ts
import { WsNode } from "robot-bus"; // Node 入口也导出 WsNode
```

### WebSocket 帧（V1）

路径：`ws://<host>:<port>/ws`（HTTPS 站点用 `wss://`）。每条连接承载一次 RPC：

| type | 值 | 含义 |
|------|----|------|
| REQUEST | 1 | 首帧：method + protobuf 请求体 |
| DATA | 2 | 响应 / 流消息 payload |
| CANCEL | 3 | 客户端软取消（SendGoal：提交 cancel，连接保持至 RESULT；Subscribe：停订） |
| TRAILER | 4 | `u32 status` + UTF-8 message（0 = OK） |

method 示例：`robot_bus_interface.grpc.v1.MessageGateway/Subscribe`（以及 Publish / ServiceGateway/Call / ActionGateway/SendGoal）。业务 payload 为 gateway protobuf（method 名仍含历史 `.grpc.v1` 包路径）。

## Action GoalHandle

Node.js（ZMQ）与浏览器（WebSocket RPC）的 action client 采用同一套 ROS 2 风格语义：`sendGoal` 立即返回 `GoalHandle`，实时 feedback 交给 callback，result 独立等待。

```ts
const action = node.createActionClient("/navigate");
const goal = action.sendGoal(goalPayload, {
  onFeedback: (feedback) => console.log("feedback", feedback),
});

const result = await goal.result();
// goal.cancel(); // best-effort，不表示服务端已确认
```

Node.js native 的 `timeoutSeconds` 覆盖整个 goal 生命周期；raw client 的 callback 收到 `ActionEvent`。通过 protobuf 类型创建的 `TypedActionClient` 会在 feedback 到达时实时解码，并让 `result()` 返回解码后的 result：

```ts
const action = node.createActionClient("/navigate", Goal, Feedback, Result);
const goal = action.sendGoal(goalMessage, {
  goalId: "navigate-1",
  timeoutSeconds: 30,
  onFeedback: (feedback) => console.log(feedback),
});
const result = await goal.result();
```

`goal.cancel()` 传输行为：

- **WebSocket RPC（浏览器）**：在同一条连接上发显式 **CANCEL** 帧，连接保持打开，继续收 `FEEDBACK` / `RESULT`（与 ZMQ 显式取消同语义）。若连接**真正断开**，服务端仍会提交 cancel 并放弃会话。
- **原生 WebSocket RPC**：与浏览器相同，发 CANCEL 帧。
- **ZMQ**：发送显式 `CANCEL` 帧。

三者都不提供服务端取消确认。底层 RPC 为 `ActionGateway.SendGoal`：一元 goal request，服务端流式返回 `FEEDBACK`，最终返回 `RESULT`。

## Protobuf 消息

磁盘上生成代码在 `bindings/typescript/generated/`（`just gen-typescript`，protoc **35.1**；**gitignored**，随 npm 包发布）；对外导入路径与 Python / Rust 对齐（无 `generated` 段）：

```ts
import { String$ } from "robot-bus/std_msgs/msg/v1/primitives.js";
import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";
```

| 语言 | 示例 |
|------|------|
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| Rust | `robot_bus::sensor_msgs::msg::v1::Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |

网关 stub：`robot-bus/robot_bus_interface/grpc/v1/*.client.js`。

改 `proto/` 后：

```bash
just gen-typescript
# 或 python3 scripts/generate_typescript_msgs.py
```

## 本地开发

```bash
just gen-typescript
just ts-dev
just test-typescript
```

## 发布

在 GitHub 上写 Release 说明并 Publish（tag 版本须与 `Cargo.toml`、`bindings/python/pyproject.toml`、`bindings/typescript/package.json` 一致）后，[`.github/workflows/publish-npm.yml`](../../.github/workflows/publish-npm.yml) 用 `secrets.NPM_TOKEN` 发布到 npm。
