import assert from "node:assert/strict";
import { afterEach, describe, it } from "node:test";
import {
  ACTION_KIND_RESULT,
  OPCODE_CALL,
  OPCODE_PUBLISH,
  OPCODE_SEND_GOAL,
  OPCODE_SUBSCRIBE,
  WsSession,
  __setWebSocketForTests,
  decodeActionData,
  decodeFrame,
  decodeSubscribeData,
  encodeActionData,
  encodeFrame,
  encodeSubscribeData,
  httpUrlToWsRpc,
} from "../src/ws-rpc.js";

describe("ws-rpc framing V3", () => {
  it("round-trips REQUEST opcodes / DATA / CANCEL / TRAILER", () => {
    const req = encodeFrame({
      type: "request",
      streamId: 1,
      header: { opcode: OPCODE_PUBLISH, topic: "/cmd" },
      body: new Uint8Array([1, 2, 3]),
    });
    const decoded = decodeFrame(req);
    assert.equal(decoded.type, "request");
    if (decoded.type === "request") {
      assert.equal(decoded.streamId, 1);
      assert.equal(decoded.header.opcode, OPCODE_PUBLISH);
      if (decoded.header.opcode === OPCODE_PUBLISH) {
        assert.equal(decoded.header.topic, "/cmd");
      }
      assert.deepEqual(Array.from(decoded.body), [1, 2, 3]);
    }

    const sub = encodeFrame({
      type: "request",
      streamId: 3,
      header: { opcode: OPCODE_SUBSCRIBE, topic: "/imu", qosDepth: 8 },
      body: new Uint8Array(),
    });
    const s = decodeFrame(sub);
    assert.equal(s.type, "request");
    if (s.type === "request" && s.header.opcode === OPCODE_SUBSCRIBE) {
      assert.equal(s.header.topic, "/imu");
      assert.equal(s.header.qosDepth, 8);
    }

    const call = encodeFrame({
      type: "request",
      streamId: 5,
      header: {
        opcode: OPCODE_CALL,
        serviceName: "svc.echo",
        timeoutMs: 1000,
        requestId: "r1",
      },
      body: new Uint8Array([9]),
    });
    const c = decodeFrame(call);
    assert.equal(c.type, "request");
    if (c.type === "request" && c.header.opcode === OPCODE_CALL) {
      assert.equal(c.header.serviceName, "svc.echo");
      assert.equal(c.header.timeoutMs, 1000);
      assert.equal(c.header.requestId, "r1");
      assert.deepEqual(Array.from(c.body), [9]);
    }

    const goal = encodeFrame({
      type: "request",
      streamId: 7,
      header: {
        opcode: OPCODE_SEND_GOAL,
        actionName: "act.nav",
        goalId: "g1",
        timeoutMs: 5000,
      },
      body: new Uint8Array([7]),
    });
    const g = decodeFrame(goal);
    assert.equal(g.type, "request");
    if (g.type === "request" && g.header.opcode === OPCODE_SEND_GOAL) {
      assert.equal(g.header.actionName, "act.nav");
      assert.equal(g.header.goalId, "g1");
      assert.equal(g.header.timeoutMs, 5000);
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
    const k = decodeFrame(cancel);
    assert.equal(k.type, "cancel");
    if (k.type === "cancel") assert.equal(k.streamId, 5);

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

  it("round-trips Subscribe DATA topic header and Action DATA kind", () => {
    const inner = encodeSubscribeData("ws.sub", new Uint8Array([1, 2]));
    const decoded = decodeSubscribeData(inner);
    assert.equal(decoded.topic, "ws.sub");
    assert.deepEqual(Array.from(decoded.payload), [1, 2]);

    const act = encodeActionData(ACTION_KIND_RESULT, new Uint8Array([3]));
    const ad = decodeActionData(act);
    assert.equal(ad.kind, ACTION_KIND_RESULT);
    assert.deepEqual(Array.from(ad.body), [3]);
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
    const pending = session.unary(
      { opcode: OPCODE_PUBLISH, topic: "/x" },
      new Uint8Array([1]),
    );
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
