/**
 * In-process Node broker start (needs napi addon: `npm run build:native`).
 * Skips when the native binary is missing. Set ROBOT_BUS_REQUIRE_NATIVE=1 to fail.
 */

import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { loadNativeOrSkip } from "./load-native.js";

const ephemeralTcp = {
  messageXsubBind: "tcp://127.0.0.1:0",
  messageXpubBind: "tcp://127.0.0.1:0",
  serviceFrontendBind: "tcp://127.0.0.1:0",
  serviceBackendBind: "tcp://127.0.0.1:0",
  actionFrontendBind: "tcp://127.0.0.1:0",
  actionBackendBind: "tcp://127.0.0.1:0",
  apiListen: "127.0.0.1:0",
};

describe("RobotBusBroker.start", () => {
  it("honors apiListen and serves the web console", async (t) => {
    const native = await loadNativeOrSkip(t, ["RobotBusBroker"]);
    if (!native) {
      return;
    }

    const broker = native.RobotBusBroker.start({
      ...ephemeralTcp,
      tcpOnly: true,
    });

    try {
      assert.match(broker.apiListen, /^127\.0\.0\.1:[1-9]\d*$/);
      assert.equal(broker.consoleListen, broker.apiListen);
      const res = await fetch(`http://${broker.apiListen}/`);
      assert.equal(res.status, 200);
      const html = await res.text();
      assert.match(html, /<html|<!doctype/i);
    } finally {
      broker.stop();
    }
  });
});
