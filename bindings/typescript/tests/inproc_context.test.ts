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
        apiListen: `127.0.0.1:${await freePort()}`,
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

  it("returns an action handle and streams feedback", async () => {
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
        apiListen: `127.0.0.1:${await freePort()}`,
        tcpOnly: false,
        noConsole: true,
      },
      ctx,
    );

    const server = native.Node.inprocWithContext(ctx, "inproc-action-server");
    try {
      server.createActionServer("/inproc/action", (...args: Array<Buffer | null>) => {
        const payload = args.at(-1);
        assert.ok(payload);
        if (payload.length === 1) {
          return [
            { phase: "FEEDBACK", body: Buffer.from([payload[0] + 1]) },
            { phase: "RESULT", body: Buffer.from([payload[0] + 2]) },
          ];
        }
        return [
          { phase: "FEEDBACK", body: Buffer.from(`step:${payload.toString()}`) },
          { phase: "RESULT", body: Buffer.from(`done:${payload.toString()}`) },
        ];
      });
      server.start();
      await sleep(150);

      const feedback: string[] = [];
      const clientNode = native.Node.inprocWithContext(ctx, "inproc-action-client");
      const client = clientNode.createActionClient("/inproc/action");
      const handle = client.sendGoal(Buffer.from("fly"), {
        goalId: "native-goal-1",
        timeoutSeconds: 5,
        onFeedback: (event) => feedback.push(event.body.toString()),
      });

      assert.equal(handle.goalId, "native-goal-1");
      assert.equal(handle.actionName, "/inproc/action");
      const resultPromise = handle.result();
      assert.ok(resultPromise instanceof Promise);

      const result = await resultPromise;
      assert.equal(result.kind, "RESULT");
      assert.equal(result.body.toString(), "done:fly");
      assert.deepEqual(feedback, ["step:fly"]);

      const { Node } = await import("../src/index.node.js");
      const byteType = {
        typeName: "test.Byte",
        create: (value: Partial<{ value: number }> = {}) => ({
          value: value.value ?? 0,
        }),
        toBinary: (value: { value: number }) => new Uint8Array([value.value]),
        fromBinary: (bytes: Uint8Array) => ({ value: bytes[0] }),
      };
      const typedFeedback: number[] = [];
      const typedClient = new Node(clientNode).createActionClient(
        "/inproc/action",
        byteType,
        byteType,
        byteType,
      );
      const typedHandle = typedClient.sendGoal(
        { value: 7 },
        { onFeedback: (value) => typedFeedback.push(value.value) },
      );
      const typedResult = await typedHandle.result();
      assert.deepEqual(typedFeedback, [8]);
      assert.deepEqual(typedResult, { value: 9 });
    } finally {
      server.stop();
      server.wait();
      broker.stop();
    }
  });
});
