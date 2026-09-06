import {
  WsNode,
  WsTopicPublisher,
  TypedWsTopicPublisher,
  coalesceSubscribeFilters,
  qosDepthForFilter,
} from "../src/ws-node.js";
import { encode, type MessageType } from "../src/typed.js";
import { __setWebSocketForTests } from "../src/ws-rpc.js";
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
    assert.equal(WsNode.ws("a").url, "http://127.0.0.1:15570");
    assert.equal(WsNode.wsAt("a", "http://example:15570/").url, "http://example:15570");
  });
});

class FakeWebSocket {
  static instances: FakeWebSocket[] = [];
  readyState = 0;
  binaryType = "arraybuffer";
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  onmessage: (() => void) | null = null;
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
  send(_data: Uint8Array): void {}
}

describe("WsNode connectionState", () => {
  afterEach(() => {
    __setWebSocketForTests(undefined);
    FakeWebSocket.instances = [];
  });

  it("tracks reconnecting when the socket closes", async () => {
    __setWebSocketForTests(FakeWebSocket as unknown as typeof WebSocket);
    const node = WsNode.wsAt("n", "http://127.0.0.1:15570");
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
