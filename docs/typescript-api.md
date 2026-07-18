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
| 浏览器 | gRPC-Web 客户端 | 仅订阅 / 调 service / action |

`console/` Web UI **不是**本 SDK。

## Broker

与 Rust / Python 相同：先起 broker，再跑业务代码。

```bash
# Rust / Python CLI
robot-bus-broker --grpc-listen 0.0.0.0:15770 --tcp-only
```

Node 进程内：

```ts
import { RobotBusBroker } from "robot-bus";

const broker = RobotBusBroker.start({
  grpcListen: "0.0.0.0:15770",
  tcpOnly: true,
  noConsole: true,
});
// …业务…
broker.stop();
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

gRPC 客户端模式（不启 ZMQ）：

```ts
const node = Node.grpc("web-client");
// 或 Node.grpcAt("web-client", "http://127.0.0.1:15770");
```

| 支持 | 不支持（gRPC 模式） |
|------|---------------------|
| `createSubscription` | `createPublisher` |
| `createClient` | `createService` |
| `createActionClient` | `createActionServer` |
| `createTimer`、`spin` / `shutdown` | — |

## 浏览器（gRPC-Web）

```ts
import { Node } from "robot-bus";
// bundler 自动解析到 browser 入口

const node = Node.grpc("browser-client");
node.createSubscription("/robot1/imu", (topic, payload) => {
  console.log(topic, payload);
});
node.start(); // 或 node.spin()
```

浏览器下 `createPublisher` / `createService` / `createActionServer` 会抛错。

也可显式使用：

```ts
import { GrpcNode } from "robot-bus"; // Node 入口也导出 GrpcNode
```

## Protobuf 消息

磁盘上生成代码在 `bindings/typescript/generated/`（`just gen-typescript`，protoc **35.1**）；对外导入路径与 Python / Rust 对齐（无 `generated` 段）：

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

打 `v*` tag（版本须与 `Cargo.toml`、`bindings/python/pyproject.toml`、`bindings/typescript/package.json` 一致）后，[`.github/workflows/publish-npm.yml`](../.github/workflows/publish-npm.yml) 用 `secrets.NPM_TOKEN` 发布到 npm。
