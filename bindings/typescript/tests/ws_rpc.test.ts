import assert from "node:assert/strict";
import { afterEach, describe, it } from "node:test";
import {
  METHOD_PUBLISH,
  WsSession,
  __setWebSocketForTests,
  decodeFrame,
  encodeFrame,
  httpUrlToWsRpc,
} from "../src/ws-rpc.js";

describe("ws-rpc framing V2", () => {
  it("round-trips REQUEST / DATA / CANCEL / TRAILER with streamId", () => {
    const req = encodeFrame({
      type: "request",
      streamId: 1,
      method: METHOD_PUBLISH,
      payload: new Uint8Array([1, 2, 3]),
    });
    const decoded = decodeFrame(req);
    assert.equal(decoded.type, "request");
    if (decoded.type === "request") {
      assert.equal(decoded.streamId, 1);
      assert.equal(decoded.method, METHOD_PUBLISH);
      assert.deepEqual(Array.from(decoded.payload), [1, 2, 3]);
    }

    const data = encodeFrame({
      type: "data",
      streamId: 3,
      payload: new Uint8Array([9]),
    });
    const d = decodeFrame(data);
    assert.equal(d.type, "data");
    if (d.type === "data") {
      assert.equal(d.streamId, 3);
      assert.deepEqual(Array.from(d.payload), [9]);
    }

    const cancel = encodeFrame({ type: "cancel", streamId: 5 });
    const c = decodeFrame(cancel);
    assert.equal(c.type, "cancel");
    if (c.type === "cancel") assert.equal(c.streamId, 5);

    const tr = encodeFrame({
      type: "trailer",
      streamId: 7,
      status: 0,
      message: "ok",
    });
    const t = decodeFrame(tr);
    assert.equal(t.type, "trailer");
    if (t.type === "trailer") {
      assert.equal(t.streamId, 7);
      assert.equal(t.status, 0);
      assert.equal(t.message, "ok");
    }
  });

  it("maps http base URL to /ws", () => {
    assert.equal(httpUrlToWsRpc("http://127.0.0.1:15570"), "ws://127.0.0.1:15570/ws");
    assert.equal(httpUrlToWsRpc("https://example.com:443/"), "wss://example.com:443/ws");
    assert.equal(httpUrlToWsRpc("ws://127.0.0.1:15570/ws"), "ws://127.0.0.1:15570/ws");
  });

  it("round-trips PING / PONG", () => {
    const ping = encodeFrame({ type: "ping", streamId: 0 });
    const p = decodeFrame(ping);
    assert.equal(p.type, "ping");
    if (p.type === "ping") assert.equal(p.streamId, 0);

    const pong = encodeFrame({ type: "pong", streamId: 0 });
    const g = decodeFrame(pong);
    assert.equal(g.type, "pong");
    if (g.type === "pong") assert.equal(g.streamId, 0);
  });
});

class FakeWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;
  static instances: FakeWebSocket[] = [];
  readyState = FakeWebSocket.CONNECTING;
  binaryType = "arraybuffer";
  sent: Uint8Array[] = [];
  onopen: ((ev?: unknown) => void) | null = null;
  onclose: ((ev?: unknown) => void) | null = null;
  onerror: ((ev?: unknown) => void) | null = null;
  onmessage: ((ev: { data: ArrayBuffer }) => void) | null = null;
  constructor(readonly url: string) {
    FakeWebSocket.instances.push(this);
    queueMicrotask(() => this.open());
  }
  open(): void {
    if (this.readyState === FakeWebSocket.OPEN) return;
    this.readyState = FakeWebSocket.OPEN;
    this.onopen?.();
  }
  close(): void {
    if (this.readyState === FakeWebSocket.CLOSED) return;
    this.readyState = FakeWebSocket.CLOSED;
    this.onclose?.({ code: 1000 });
  }
  send(data: Uint8Array): void {
    this.sent.push(data instanceof Uint8Array ? data : new Uint8Array(data));
  }
  reply(data: Uint8Array): void {
    const copy = data.slice();
    this.onmessage?.({ data: copy.buffer });
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

describe("WsSession reconnect", () => {
  afterEach(() => {
    __setWebSocketForTests(undefined);
    FakeWebSocket.instances = [];
  });

  it("reconnects after close and fails in-flight unary", async () => {
    __setWebSocketForTests(FakeWebSocket as unknown as typeof WebSocket);
    const session = new WsSession("http://127.0.0.1:15570", {
      backoffInitialMs: 20,
      backoffMaxMs: 50,
      pingIntervalMs: 10_000,
    });
    const opened = await session.waitUntilOpen(500);
    assert.equal(opened, true);
    assert.equal(FakeWebSocket.instances.length, 1);

    const first = FakeWebSocket.instances[0]!;
    const pending = session.unary(METHOD_PUBLISH, new Uint8Array([1]));
    first.close();
    await assert.rejects(pending, /websocket closed/);

    await sleep(80);
    assert.ok(FakeWebSocket.instances.length >= 2);
    session.close();
  });

  it("stop reconnecting after close()", async () => {
    __setWebSocketForTests(FakeWebSocket as unknown as typeof WebSocket);
    const session = new WsSession("http://127.0.0.1:15570", {
      backoffInitialMs: 10,
      backoffMaxMs: 20,
      pingIntervalMs: 10_000,
    });
    assert.equal(await session.waitUntilOpen(500), true);
    session.close();
    const n = FakeWebSocket.instances.length;
    FakeWebSocket.instances.at(-1)?.close();
    await sleep(80);
    assert.equal(FakeWebSocket.instances.length, n);
  });
});
