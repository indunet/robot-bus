import {
  WsNode,
  WsTopicPublisher,
  TypedWsTopicPublisher,
  WsRpcError,
  coalesceSubscribeFilters,
  qosDepthForFilter,
} from "../src/ws-node.js";
import { encode, type MessageType } from "../src/typed.js";
import {
  ACTION_KIND_RESULT,
  __setWebSocketForTests,
  decodeFrame,
  encodeActionData,
  encodeFrame,
  encodeSubscribeData,
} from "../src/ws-rpc.js";
import assert from "node:assert/strict";
import { afterEach, describe, it } from "node:test";

describe("coalesceSubscribeFilters", () => {
  it("multiplexes console /robot_bus/* topics onto one prefix stream", () => {
    assert.deepEqual(
      coalesceSubscribeFilters([
        "/robot_bus/status",
        "/robot_bus/topics",
        "/robot_bus/services",
        "/robot_bus/actions",
        "/robot_bus/topology",
        "/robot_bus/events",
        "/robot_bus/bridges",
      ]),
      ["/robot_bus/"],
    );
  });

  it("keeps unrelated topics on separate streams", () => {
    assert.deepEqual(
      coalesceSubscribeFilters(["/robot1/imu", "/robot_bus/status"]),
      ["/robot1/imu", "/robot_bus/status"],
    );
  });

  it("passes through a single topic", () => {
    assert.deepEqual(coalesceSubscribeFilters(["/robot_bus/tank/pose"]), [
      "/robot_bus/tank/pose",
    ]);
  });
});

describe("qosDepthForFilter", () => {
  it("takes the max KeepLast of topics covered by a coalesced prefix", () => {
    const qos = new Map([
      ["/robot_bus/status", 4],
      ["/robot_bus/topics", 16],
      ["/robot1/imu", 8],
    ]);
    assert.equal(qosDepthForFilter("/robot_bus/", qos), 16);
    assert.equal(qosDepthForFilter("/robot1/imu", qos), 8);
    assert.equal(qosDepthForFilter("/other", qos), 0);
  });
});

describe("WsNode capability guards", () => {
  it("accepts only a numeric KeepLast depth and shares equivalent default depths", () => {
    const node = WsNode.ws("keep-last");
    node.createSubscription("/same", () => {});
    node.createSubscription("/same", () => {}, 64);
    node.createSubscription("/same", () => {}, -1);
    assert.throws(() => node.createSubscription("/same", () => {}, 1), /conflicting KeepLast/);
    for (const depth of [NaN, Infinity, 1.5, { overflow: "latest" }, { depth: 3 }]) {
      assert.throws(() => node.createSubscription("/invalid", () => {}, depth as number), /KeepLast depth must be an integer/);
    }
  });
  it("rejects service / action servers", () => {
    const node = WsNode.ws("test");
    assert.throws(() => node.createService("/s", () => new Uint8Array()), /not available/);
    assert.throws(() => node.createActionServer("/a", () => []), /not available/);
  });

  it("createPublisher returns raw and typed publishers", () => {
    const node = WsNode.ws("test");
    const raw = node.createPublisher("/t");
    assert.ok(raw instanceof WsTopicPublisher);
    assert.equal(raw.topic, "/t");

    const FakeType = {
      typeName: "fake.v1.Msg",
      create: (v?: object) => (v ?? {}) as object,
      toBinary: () => new Uint8Array([1, 2, 3]),
      fromBinary: () => ({}),
    } as MessageType<object>;
    const typed = node.createPublisher("/typed", FakeType);
    assert.ok(typed instanceof TypedWsTopicPublisher);
    assert.equal(typed.topic, "/typed");
    assert.deepEqual(Array.from(encode(FakeType, {})), [1, 2, 3]);
  });

  it("default and custom urls", () => {
    assert.equal(WsNode.ws("a").url, "http://127.0.0.1:15560");
    assert.equal(WsNode.wsAt("a", "http://example:15560/").url, "http://example:15560");
  });
});

class FakeWebSocket {
  static instances: FakeWebSocket[] = [];
  readyState = 0;
  binaryType = "arraybuffer";
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  onmessage: ((ev: { data: ArrayBuffer }) => void) | null = null;
  sent: Uint8Array[] = [];
  constructor(readonly url: string) {
    FakeWebSocket.instances.push(this);
    queueMicrotask(() => {
      this.readyState = 1;
      this.onopen?.();
    });
  }
  close(): void {
    this.readyState = 3;
    this.onclose?.();
  }
  send(data: Uint8Array): void { this.sent.push(data); }
}

describe("WsNode connectionState", () => {
  afterEach(() => {
    __setWebSocketForTests(undefined);
    FakeWebSocket.instances = [];
  });

  it("keeps numeric KeepLast depths on separate streams and dispatches overlapping filters once", async () => {
    __setWebSocketForTests(FakeWebSocket as unknown as typeof WebSocket);
    const node = WsNode.ws("policies");
    const received: string[] = [];
    node.createSubscription("/robot_bus/", () => received.push("prefix"), 3);
    node.createSubscription("/robot_bus/pose", () => received.push("pose"), 1);
    node.createSubscription("/legacy", () => {}, 8);
    assert.throws(() => node.createSubscription("/robot_bus/pose", () => {}, 8), /conflicting/);
    try {
      node.start();
      await new Promise(resolve => setTimeout(resolve, 30));
      const socket = FakeWebSocket.instances[0];
      const requests = socket.sent.map(decodeFrame).filter(f => f.type === "request" && (f.header.opcode === 1 || f.header.opcode === 5));
      assert.equal(requests.length, 3);
      assert.deepEqual(requests.map(r => r.header), [
        { opcode: 5, topic: "/robot_bus/", qosDepth: 3, overflow: 1 },
        { opcode: 5, topic: "/robot_bus/pose", qosDepth: 1, overflow: 1 },
        { opcode: 5, topic: "/legacy", qosDepth: 8, overflow: 1 },
      ]);
      for (const request of requests.slice(0, 2)) {
        const bytes = encodeFrame({ type: "data", streamId: request.streamId, payload: encodeSubscribeData("/robot_bus/pose", new Uint8Array([1])) });
        socket.onmessage?.({ data: bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer });
      }
      await new Promise(resolve => setTimeout(resolve, 10));
      assert.deepEqual(received, ["prefix", "pose"]);
    } finally { node.shutdown(); }
  });

  it("tracks reconnecting when the socket closes", async () => {
    __setWebSocketForTests(FakeWebSocket as unknown as typeof WebSocket);
    const node = WsNode.wsAt("n", "http://127.0.0.1:15560");
    const states: string[] = [];
    node.addOnConnectionEvent((_o, next) => states.push(next));
    assert.equal(await node.waitForBroker(1), true);
    assert.equal(node.connectionState(), "connected");
    FakeWebSocket.instances[0]?.close();
    await new Promise((r) => setTimeout(r, 30));
    assert.equal(node.connectionState(), "reconnecting");
    assert.ok(states.includes("connected"));
    assert.ok(states.includes("reconnecting"));
    node.shutdown();
  });
});

describe("WsNode action terminal errors", () => {
  afterEach(() => {
    __setWebSocketForTests(undefined);
    FakeWebSocket.instances = [];
  });

  it("surfaces cancel trailers as WsRpcError instead of a generic missing result", async () => {
    __setWebSocketForTests(FakeWebSocket as unknown as typeof WebSocket);
    const node = WsNode.ws("act");
    const handle = node.sendGoal("/navigate", new Uint8Array([1]), { goalId: "g1" });
    await new Promise((r) => setTimeout(r, 30));
    const socket = FakeWebSocket.instances[0];
    assert.ok(socket);
    const request = socket.sent.map(decodeFrame).find((f) => f.type === "request");
    assert.equal(request?.type, "request");
    const streamId = request && request.type === "request" ? request.streamId : 1;
    const pending = handle.result();
    const bytes = encodeFrame({ type: "trailer", streamId, status: 1, message: "cancelled 'motion'" });
    socket.onmessage?.({ data: bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer });
    await assert.rejects(pending, (err: unknown) => {
      assert.ok(err instanceof WsRpcError);
      assert.equal(err.code, "cancelled");
      assert.equal(err.status, 1);
      return true;
    });
    node.shutdown();
  });

  it("treats CANCELLED result bodies as errors even if the trailer is OK", async () => {
    __setWebSocketForTests(FakeWebSocket as unknown as typeof WebSocket);
    const node = WsNode.ws("act-body");
    const handle = node.sendGoal("/navigate", new Uint8Array([1]), { goalId: "g2" });
    await new Promise((r) => setTimeout(r, 30));
    const socket = FakeWebSocket.instances[0];
    assert.ok(socket);
    const request = socket.sent.map(decodeFrame).find((f) => f.type === "request");
    const streamId = request && request.type === "request" ? request.streamId : 1;
    const pending = handle.result();
    const body = Uint8Array.from([...new TextEncoder().encode("ACTION_ABORTED"), 0, ...new TextEncoder().encode("stopped")]);
    const data = encodeFrame({ type: "data", streamId, payload: encodeActionData(ACTION_KIND_RESULT, body) });
    socket.onmessage?.({ data: data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer });
    const trailer = encodeFrame({ type: "trailer", streamId, status: 0, message: "" });
    socket.onmessage?.({ data: trailer.buffer.slice(trailer.byteOffset, trailer.byteOffset + trailer.byteLength) as ArrayBuffer });
    await assert.rejects(pending, (err: unknown) => {
      assert.ok(err instanceof WsRpcError);
      assert.equal(err.code, "aborted");
      return true;
    });
    node.shutdown();
  });
});
