import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  WsNode,
  WsTopicPublisher,
  TypedWsTopicPublisher,
  coalesceSubscribeFilters,
} from "../src/ws-node.js";
import { encode, type MessageType } from "../src/typed.js";

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
