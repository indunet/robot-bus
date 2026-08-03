/**
 * TF buffer / listener smoke (needs napi addon from `just ts-dev`).
 */

import assert from "node:assert/strict";
import { createServer } from "node:net";
import { describe, it } from "node:test";
import { setTimeout as sleep } from "node:timers/promises";
import { TransformStamped } from "../generated/geometry_msgs/msg/v1/stamped.js";
import { TFMessage } from "../generated/tf2_msgs/msg/v1/tf_message.js";
import {
  createTfBuffer,
  Node,
  RobotBusBroker,
  TfListener,
  TransformBroadcaster,
} from "../src/index.node.js";

async function freePort(): Promise<number> {
  return await new Promise((resolve, reject) => {
    const server = createServer();
    server.listen(0, "127.0.0.1", () => {
      const addr = server.address();
      if (!addr || typeof addr === "string") {
        server.close();
        reject(new Error("no port"));
        return;
      }
      const port = addr.port;
      server.close((err) => (err ? reject(err) : resolve(port)));
    });
  });
}

function staticEdge(
  parent: string,
  child: string,
  x: number,
  y: number,
): TFMessage {
  return TFMessage.create({
    transforms: [
      TransformStamped.create({
        header: { frameId: parent },
        childFrameId: child,
        transform: {
          translation: { x, y, z: 0 },
          rotation: { x: 0, y: 0, z: 0, w: 1 },
        },
      }),
    ],
  });
}

describe("tf lookup", () => {
  it("offline buffer", () => {
    let buf;
    try {
      buf = createTfBuffer();
    } catch {
      return; // native addon missing
    }
    buf.setTransformMsg(TFMessage, staticEdge("base_link", "camera", 1, 0), true);
    assert.equal(buf.canTransform("base_link", "camera"), true);
    const t = buf.lookupTransform("base_link", "camera", TransformStamped);
    assert.equal(t.childFrameId, "camera");
    assert.ok(Math.abs((t.transform?.translation?.x ?? 0) - 1) < 1e-9);
  });

  it("listener against broker", async () => {
    let broker: InstanceType<typeof RobotBusBroker>;
    try {
      broker = RobotBusBroker.start({
        messageXsubBind: `tcp://127.0.0.1:${await freePort()}`,
        messageXpubBind: `tcp://127.0.0.1:${await freePort()}`,
        serviceFrontendBind: `tcp://127.0.0.1:${await freePort()}`,
        serviceBackendBind: `tcp://127.0.0.1:${await freePort()}`,
        actionFrontendBind: `tcp://127.0.0.1:${await freePort()}`,
        actionBackendBind: `tcp://127.0.0.1:${await freePort()}`,
        grpcListen: `127.0.0.1:${await freePort()}`,
        tcpOnly: true,
        noConsole: true,
        noDiscovery: true,
      });
    } catch {
      return;
    }

    try {
      const node = new Node(
        "ts-tf",
        "localhost",
        "tcp",
        undefined,
        broker.messageXsubBind,
        broker.messageXpubBind,
        broker.serviceFrontendBind,
        broker.serviceBackendBind,
        broker.actionBackendBind,
        broker.actionFrontendBind,
      );
      const listener = new TfListener(node);
      const buf = listener.buffer();
      const br = TransformBroadcaster.fromTyped(
        node.createPublisher("/tf_static", TFMessage),
        TFMessage,
      );
      node.start();
      await sleep(200);
      br.send(staticEdge("odom", "base_link", 0, 2));
      const deadline = Date.now() + 3000;
      while (!buf.canTransform("odom", "base_link") && Date.now() < deadline) {
        await sleep(20);
      }
      assert.equal(buf.canTransform("odom", "base_link"), true);
      const t = buf.lookupTransform("odom", "base_link", TransformStamped);
      assert.ok(Math.abs((t.transform?.translation?.y ?? 0) - 2) < 1e-9);
      node.shutdown();
      node.wait();
    } finally {
      broker.stop();
    }
  });
});
