/**
 * Same-process inproc requires a shared Context with the embedded broker.
 *
 * Needs the napi addon: `npm run build:native` (or `just ts-dev`).
 * Skips cleanly when the native binary is missing (CI smoke without ZMQ).
 */

import assert from "node:assert/strict";
import { createServer } from "node:net";
import { describe, it } from "node:test";
import { setTimeout as sleep } from "node:timers/promises";
import type { NativeBinding } from "../src/native.js";

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

async function tryLoadNative(): Promise<NativeBinding | null> {
  try {
    const { loadNative } = await import("../src/native.js");
    return loadNative();
  } catch {
    return null;
  }
}

describe("inproc shared Context", () => {
  it("pubsub with shared context", async () => {
    const native = await tryLoadNative();
    if (!native?.Context || !native.RobotBusBroker || !native.Node) {
      return;
    }

    const ctx = new native.Context();
    const broker = native.RobotBusBroker.start(
      {
        messageXsubBind: `tcp://127.0.0.1:${await freePort()}`,
        messageXpubBind: `tcp://127.0.0.1:${await freePort()}`,
        serviceFrontendBind: `tcp://127.0.0.1:${await freePort()}`,
        serviceBackendBind: `tcp://127.0.0.1:${await freePort()}`,
        actionFrontendBind: `tcp://127.0.0.1:${await freePort()}`,
        actionBackendBind: `tcp://127.0.0.1:${await freePort()}`,
        grpcListen: `127.0.0.1:${await freePort()}`,
        tcpOnly: false,
        noConsole: true,
      },
      ctx,
    );

    try {
      await sleep(150);

      const hits: Buffer[] = [];
      const sub = native.Node.inprocWithContext(ctx, "inproc-sub");
      sub.createSubscription("/inproc/demo", (topic, payload) => {
        hits.push(Buffer.from(payload as Buffer));
      });
      sub.start();
      await sleep(100);

      const pub = native.Node.inprocWithContext(ctx, "inproc-pub");
      const topic = pub.createPublisher("/inproc/demo");
      const deadline = Date.now() + 5000;
      while (hits.length === 0 && Date.now() < deadline) {
        topic.publish(Buffer.from("hello-inproc"));
        await sleep(20);
      }

      assert.ok(hits.length >= 1, "expected at least one inproc message");
      assert.equal(hits[0].toString("utf8"), "hello-inproc");

      sub.shutdown();
      sub.stop();
      sub.wait();
    } finally {
      broker.stop();
    }
  });
});
