/**
 * In-process Node broker start (needs napi addon: `npm run build:native`).
 * Skips when the native binary is missing.
 */

import assert from "node:assert/strict";
import { createServer } from "node:net";
import { describe, it } from "node:test";
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

describe("RobotBusBroker.start", () => {
  it("honors apiListen and serves the web console", async () => {
    const native = await tryLoadNative();
    if (!native?.RobotBusBroker) {
      return;
    }

    const port = await freePort();
    const broker = native.RobotBusBroker.start({
      messageXsubBind: `tcp://127.0.0.1:${await freePort()}`,
      messageXpubBind: `tcp://127.0.0.1:${await freePort()}`,
      serviceFrontendBind: `tcp://127.0.0.1:${await freePort()}`,
      serviceBackendBind: `tcp://127.0.0.1:${await freePort()}`,
      actionFrontendBind: `tcp://127.0.0.1:${await freePort()}`,
      actionBackendBind: `tcp://127.0.0.1:${await freePort()}`,
      apiListen: `127.0.0.1:${port}`,
      tcpOnly: true,
    });

    try {
      assert.equal(broker.apiListen, `127.0.0.1:${port}`);
      assert.equal(broker.consoleListen, `127.0.0.1:${port}`);
      const res = await fetch(`http://127.0.0.1:${port}/`);
      assert.equal(res.status, 200);
      const html = await res.text();
      assert.match(html, /<html|<!doctype/i);
    } finally {
      broker.stop();
    }
  });
});
