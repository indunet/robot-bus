import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  METHOD_PUBLISH,
  decodeFrame,
  encodeFrame,
  httpUrlToWsRpc,
} from "../src/ws-rpc.js";

describe("ws-rpc framing", () => {
  it("round-trips REQUEST and TRAILER", () => {
    const req = encodeFrame({
      type: "request",
      method: METHOD_PUBLISH,
      payload: new Uint8Array([1, 2, 3]),
    });
    const decoded = decodeFrame(req);
    assert.equal(decoded.type, "request");
    if (decoded.type === "request") {
      assert.equal(decoded.method, METHOD_PUBLISH);
      assert.deepEqual(Array.from(decoded.payload), [1, 2, 3]);
    }

    const tr = encodeFrame({ type: "trailer", status: 0, message: "ok" });
    const t = decodeFrame(tr);
    assert.equal(t.type, "trailer");
    if (t.type === "trailer") {
      assert.equal(t.status, 0);
      assert.equal(t.message, "ok");
    }
  });

  it("maps http base URL to /ws", () => {
    assert.equal(httpUrlToWsRpc("http://127.0.0.1:15570"), "ws://127.0.0.1:15570/ws");
    assert.equal(httpUrlToWsRpc("https://example.com:443/"), "wss://example.com:443/ws");
    assert.equal(httpUrlToWsRpc("ws://127.0.0.1:15570/ws"), "ws://127.0.0.1:15570/ws");
  });
});
