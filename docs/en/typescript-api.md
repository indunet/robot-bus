English | [中文](../zh/typescript-api.md)

# TypeScript API

```bash
npm install robot-bus
# Local: just ts-dev
# Equivalent: cd bindings/typescript && npm install && npm run build:native && npm run build:ts
```

Single npm package; entry is chosen automatically by runtime (`package.json` `exports`):

| Environment | Entry | Capabilities |
|------|------|------|
| Node.js | napi-rs native extension | Full ZMQ Node (publish, service/action server, local broker) |
| Browser | WebSocket RPC (`/ws`) | Subscribe / publish / call service / action (no server) |

The `console/` Web UI is **not** this SDK.

## Broker

Same as Rust / Python: start the broker first, then run application code.

```bash
# Rust / Python CLI
robot-bus-broker --grpc-listen 0.0.0.0:15570 --tcp-only
```

In-process in Node:

```ts
import { RobotBusBroker } from "robot-bus";

const broker = RobotBusBroker.start({
  grpcListen: "0.0.0.0:15570",
  tcpOnly: true,
  noConsole: true,
});
// …application…
broker.stop();
```

### HTTP discovery (Node.js native only)

Request `GET /api/v1/discover` on a known API base URL, then connect with the chosen transport:

```ts
import { Node } from "robot-bus";

const node = Node.discover("talker", {
  transport: "tcp",
  apiUrl: "http://127.0.0.1:15570",
  // optional: brokerId / timeoutSecs
});
```

The browser entry has no HTTP discover factory; use an explicit `wsUrl` (HTTP origin; the SDK connects to `ws://…/ws`).

Cross-broker (federation) uses the same string conventions as the CLI:

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

Same-process **inproc** requires a shared `Context`:

```ts
import { Context, Node, RobotBusBroker } from "robot-bus";

const ctx = new Context();
const broker = RobotBusBroker.start({ noConsole: true }, ctx);
const node = Node.inprocWithContext(ctx, "pilot");
```

tcp / ipc / ws do not require a shared Context.

## Local parameters (Node.js)

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

## Node.js (full API)

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
// createWallTimer; optional qosDepth; waitForMessage / waitForService
// listParameters() → { names, prefixes }; listAllParameters()

pub.publish(
  Imu.create({
    /* … */
  }),
);

// node.spin();
broker.stop();
```

WebSocket RPC client mode (no ZMQ):

```ts
const node = Node.ws("web-client");
// or Node.wsAt("web-client", "http://127.0.0.1:15570");
```

| Supported | Not supported (WS mode) |
|------|---------------------|
| `createSubscription` | `createService` |
| `createPublisher` | `createActionServer` |
| `createClient` | — |
| `createActionClient` | |
| `createTimer`, `spin` / `shutdown` | |

## Browser (WebSocket RPC)

Browser clients use the broker’s **`/ws`** (gRPC-like over WebSocket, **single connection multiplexed (V2 stream_id)**), not gRPC-Web.

```ts
import { Node } from "robot-bus";
// bundler resolves to browser entry automatically

const node = Node.ws("browser-client"); // default http://127.0.0.1:15570 → ws://127.0.0.1:15570/ws
const pub = node.createPublisher("/robot1/cmd");
await pub.publish(new TextEncoder().encode("go"));
node.createSubscription("/robot1/imu", (topic, payload) => {
  console.log(topic, payload);
});
node.start(); // or node.spin()
```

`createService` / `createActionServer` throw in the browser.

You can also use explicitly:

```ts
import { WsNode } from "robot-bus"; // Node entry also exports WsNode
```

### WebSocket frames (V1)

Path: `ws://<host>:<port>/ws` (use `wss://` on HTTPS sites). Each connection carries one RPC:

| type | value | meaning |
|------|----|------|
| REQUEST | 1 | First frame: method + protobuf request body |
| DATA | 2 | Response / stream message payload |
| CANCEL | 3 | Client soft cancel (SendGoal: submit cancel, connection stays open until RESULT; Subscribe: stop subscription) |
| TRAILER | 4 | `u32 status` + UTF-8 message (0 = OK) |

Example methods: `robot_bus_interfaces.grpc.v1.MessageGateway/Subscribe` (also Publish / ServiceGateway/Call / ActionGateway/SendGoal). Business payload uses gateway protobuf (method names still include the historical `.grpc.v1` package path).

## Action GoalHandle

Action clients on Node.js (ZMQ) and in the browser (WebSocket RPC) share the same ROS 2–style semantics: `sendGoal` returns a `GoalHandle` immediately, feedback is delivered to the callback in real time, and result is waited on independently.

```ts
const action = node.createActionClient("/navigate");
const goal = action.sendGoal(goalPayload, {
  onFeedback: (feedback) => console.log("feedback", feedback),
});

const result = await goal.result();
// goal.cancel(); // best-effort; does not mean the server confirmed
```

On Node.js native, `timeoutSeconds` covers the entire goal lifecycle; raw client callbacks receive `ActionEvent`. A `TypedActionClient` created with protobuf types decodes feedback in real time and makes `result()` return the decoded result:

```ts
const action = node.createActionClient("/navigate", Goal, Feedback, Result);
const goal = action.sendGoal(goalMessage, {
  goalId: "navigate-1",
  timeoutSeconds: 30,
  onFeedback: (feedback) => console.log(feedback),
});
const result = await goal.result();
```

`goal.cancel()` transport behavior:

- **WebSocket RPC (browser)**: send an explicit **CANCEL** frame on the same connection; the connection stays open and continues receiving `FEEDBACK` / `RESULT` (same semantics as ZMQ explicit cancel). If the connection **actually drops**, the server still submits cancel and abandons the session.
- **Native WebSocket RPC**: same as browser — send CANCEL frame.
- **ZMQ**: send explicit `CANCEL` frame.

None of the three provide server-side cancel acknowledgment. Underlying RPC is `ActionGateway.SendGoal`: unary goal request, server streams `FEEDBACK`, then returns `RESULT`.

## Protobuf messages

Generated code on disk lives in `bindings/typescript/generated/` (`just gen-typescript`, protoc **35.1**; **gitignored**, published with the npm package); public import paths align with Python / Rust (no `generated` segment):

```ts
import { String$ } from "robot-bus/std_msgs/msg/v1/primitives.js";
import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";
```

| Language | Example |
|------|------|
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| Rust | `robot_bus::sensor_msgs::msg::v1::Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |

Gateway stubs: `robot-bus/robot_bus_interfaces/grpc/v1/*.client.js`.

After changing `proto/`:

```bash
just gen-typescript
# or python3 scripts/generate_typescript_msgs.py
```

## Local development

```bash
just gen-typescript
just ts-dev
just test-typescript
```

## Publishing

After writing Release notes on GitHub and publishing (tag version must match `Cargo.toml`, `bindings/python/pyproject.toml`, `bindings/typescript/package.json`), [`.github/workflows/publish-npm.yml`](../../.github/workflows/publish-npm.yml) publishes to npm using `secrets.NPM_TOKEN`.
