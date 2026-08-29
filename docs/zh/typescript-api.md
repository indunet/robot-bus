[English](../en/typescript-api.md) | 中文

# TypeScript API

```bash
npm install robot-bus
# 本地：just ts-dev
# 等价：cd bindings/typescript && npm install && npm run build:native && npm run build:ts
```

单一 npm包，按运行环境自动选入口（`package.json` `exports`）：

| 环境 | 入口 | 能力 |
|------|------|------|
| Node.js | napi-rs原生扩展 | 完整 ZMQ Node（publish、service/action server、本地 broker） |
| 浏览器 | WebSocket RPC（`/ws`） | 订阅 / publish / 调 service / action（无 server） |

`console/` Web UI **不是**本 SDK。

## Broker

与 Rust / Python相同：先起 broker，再跑业务代码。

```bash
# Rust / Python CLI
robot-bus-broker --api-listen 0.0.0.0:15570 --tcp-only
```

Node进程内：

```ts
import { RobotBusBroker } from "robot-bus";

const broker = RobotBusBroker.start({
  apiListen: "0.0.0.0:15570",
  tcpOnly: true,
});
// …业务…
broker.stop();
```

### HTTP discovery（仅 Node.js原生）

对已知 API口请求 `GET /api/v1/discover`，再按所选传输连接：

```ts
import { Node } from "robot-bus";

const node = Node.discover("talker", {
  transport: "tcp",
  apiUrl: "http://127.0.0.1:15570",
  // brokerId / timeoutSecs可选
});
```

浏览器入口无 HTTP discover工厂；请用显式 `wsUrl`（HTTP原点；SDK会连 `ws://…/ws`）。

跨 broker（federation）与 CLI同款字符串约定：

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

tcp / ipc / ws不要求共享 Context。

## 本地参数（Node.js）

```ts
const node = new Node("pilot");
node.declareParameter("max_speed", 1.5);
node.declareParameter("frame_id", "base_link");
node.setParameter("max_speed", 2.0);
console.log(node.getParameter("max_speed").value); // { name, value }
console.log(node.listParameters()); // { names, prefixes }
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
const sub = node.createSubscription(
  "/robot1/imu",
  (_topic, imu) => {
    console.log(imu);
  },
  Imu,
);
// node.destroySubscription(sub)
// createWallTimer = createTimer；可选 qosDepth
// waitForMessage / client.waitForService / waitForActionServer
// listParameters() → { names, prefixes }；listAllParameters()

pub.publish(
  Imu.create({
    /* … */
  }),
);

// node.spin();
broker.stop();
```

WebSocket RPC客户端模式（不启 ZMQ）：

```ts
const node = Node.ws("web-client");
// 或 Node.wsAt("web-client", "http://127.0.0.1:15570");
```

| 支持 | 不支持（WS模式） |
|------|---------------------|
| `createSubscription` | `createService` |
| `createPublisher` | `createActionServer` |
| `createClient` | — |
| `createActionClient` | |
| `createTimer`、`spin` / `shutdown` | |

## 浏览器（WebSocket RPC）

浏览器客户端走 broker的 **`/ws`**（V3多路复用 WebSocket RPC，**一条连接多条流**），不再使用 gRPC-Web。

```ts
import { Node } from "robot-bus";
// bundler自动解析到 browser入口

const node = Node.ws("browser-client"); // 默认 http://127.0.0.1:15570 → ws://127.0.0.1:15570/ws
const pub = node.createPublisher("/robot1/cmd");
await pub.publish(new TextEncoder().encode("go"));
node.createSubscription("/robot1/imu", (topic, payload) => {
  console.log(topic, payload);
}, 10); // 可选 KeepLast depth → 网关订阅队列
node.start(); // 或 node.spin()
```

浏览器下 `createService` / `createActionServer`会抛错。

也可显式使用：

```ts
import { WsNode } from "robot-bus"; // Node入口也导出 WsNode
```

### WebSocket帧（V3多路复用）

路径：`ws://<host>:<port>/ws`（HTTPS站点用 `wss://`）。**一条连接承载多个 RPC**（`stream_id`，客户端用奇数）。会话会自动退避重连（200ms–5s）；`connectionState` / `waitForBroker`跟着这条 WebSocket，不是 HTTP 200。进行中的 Publish / Call / SendGoal **当次失败**，不自动重放；订阅在新连接上重新 Subscribe。

| type | 值 | 含义 |
|------|----|------|
| REQUEST | 1 | 首帧：`u8 opcode` + 路由头 + 原始 body |
| DATA | 2 | 流 payload（原始总线字节；Subscribe前缀话题名，SendGoal前缀 kind） |
| CANCEL | 3 | 客户端软取消（SendGoal：提交 cancel，连接保持至 RESULT；Subscribe：停订） |
| TRAILER | 4 | `u32 status` + UTF-8 message（0 = OK） |
| PING | 5 | 应用层心跳（`stream_id = 0`） |
| PONG | 6 | 心跳应答 |

opcode：`1=Subscribe`、`2=Publish`、`3=Call`、`4=SendGoal`。Publish成功只回 TRAILER（无 DATA ack）。**不兼容旧版：** 不再接受 V2 method字符串和 `TopicMessage` protobuf信封。

Node会话合同（`connection_state` / `wait_for_broker` / 自动重连）与传输无关；浏览器补的是 WebSocket实现，不是另一套业务语义。

## Action GoalHandle

Node.js（ZMQ）与浏览器（WebSocket RPC）的 action client采用同一套 ROS2风格语义：`sendGoal`立即返回 `GoalHandle`，实时 feedback交给 callback，result独立等待。

```ts
const action = node.createActionClient("/navigate");
const goal = action.sendGoal(goalPayload, {
  onFeedback: (feedback) => console.log("feedback", feedback),
});

const result = await goal.result();
// goal.cancel(); // best-effort，不表示服务端已确认
```

Node.js native的 `timeoutSeconds`覆盖整个 goal生命周期；raw client的 callback收到 `ActionEvent`。通过 protobuf类型创建的 `TypedActionClient`会在 feedback到达时实时解码，并让 `result()`返回解码后的 result：

```ts
const action = node.createActionClient("/navigate", Goal, Feedback, Result);
const goal = action.sendGoal(goalMessage, {
  goalId: "navigate-1",
  timeoutSeconds: 30,
  onFeedback: (feedback) => console.log(feedback),
});
const result = await goal.result();
```

`goal.cancel()`传输行为：

- **WebSocket RPC（浏览器）**：在同一条连接上发显式 **CANCEL** 帧，连接保持打开，继续收 `FEEDBACK` / `RESULT`（与 ZMQ显式取消同语义）。若连接**真正断开**，服务端仍会提交 cancel并放弃会话。
- **原生 WebSocket RPC**：与浏览器相同，发 CANCEL帧。
- **ZMQ**：发送显式 `CANCEL`帧。

三者都不提供服务端取消确认。SendGoal是 opcode `4`：一元 goal REQUEST，服务端流式 `FEEDBACK`后接 `RESULT`。

## Protobuf消息

磁盘上生成代码在 `bindings/typescript/generated/`（`just gen-typescript`，protoc **35.1**；**gitignored**，随 npm包发布）；对外导入路径与 Python / Rust对齐（无 `generated`段）：

```ts
import { String$ } from "robot-bus/std_msgs/msg/v1/primitives.js";
import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";
```

| 语言 | 示例 |
|------|------|
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| Rust | `robot_bus::sensor_msgs::msg::v1::Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |

网关 stub：`robot-bus/robot_bus_interfaces/grpc/v1/*.client.js`。

改 `proto/`后：

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

在 GitHub上写 Release说明并 Publish（tag版本须与 `Cargo.toml`、`bindings/python/pyproject.toml`、`bindings/typescript/package.json`一致）后，[`.github/workflows/publish-npm.yml`](../../.github/workflows/publish-npm.yml) 用 `secrets.NPM_TOKEN`发布到 npm。
