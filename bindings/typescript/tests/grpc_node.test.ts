import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  GrpcNode,
  GrpcTopicPublisher,
  TypedGrpcTopicPublisher,
} from "../src/grpc-node.js";
import { encode, type MessageType } from "../src/typed.js";

describe("GrpcNode capability guards", () => {
  it("rejects service / action servers", () => {
    const node = GrpcNode.grpc("test");
    assert.throws(() => node.createService("/s", () => new Uint8Array()), /not available/);
    assert.throws(() => node.createActionServer("/a", () => []), /not available/);
  });

  it("createPublisher returns raw and typed publishers", () => {
    const node = GrpcNode.grpc("test");
    const raw = node.createPublisher("/t");
    assert.ok(raw instanceof GrpcTopicPublisher);
    assert.equal(raw.topic, "/t");

    const FakeType = {
      typeName: "fake.v1.Msg",
      create: (v?: object) => (v ?? {}) as object,
      toBinary: () => new Uint8Array([1, 2, 3]),
      fromBinary: () => ({}),
    } as MessageType<object>;
    const typed = node.createPublisher("/typed", FakeType);
    assert.ok(typed instanceof TypedGrpcTopicPublisher);
    assert.equal(typed.topic, "/typed");
    assert.deepEqual(Array.from(encode(FakeType, {})), [1, 2, 3]);
  });

  it("exposes grpc factory urls", () => {
    assert.equal(GrpcNode.grpc("a").url, "http://127.0.0.1:15770");
    assert.equal(GrpcNode.grpcAt("a", "http://example:15770/").url, "http://example:15770");
  });
});

describe("GrpcNode console registration", () => {
  it("registers, refreshes, and unregisters browser topic endpoints", async () => {
    const originalFetch = globalThis.fetch;
    const requests: Array<{ url: string; body: Record<string, unknown> }> = [];
    globalThis.fetch = (async (input, init) => {
      requests.push({
        url: String(input),
        body: JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>,
      });
      return new Response(JSON.stringify({ ok: true }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    }) as typeof fetch;

    try {
      const FakeType = {
        typeName: "fake.v1.Msg",
        create: (v?: object) => (v ?? {}) as object,
        toBinary: () => new Uint8Array(),
        fromBinary: () => ({}),
      } as MessageType<object>;
      const node = GrpcNode.grpcAt("web_test", "http://grpc.invalid", {
        consoleUrl: "http://console.test/",
        topologyRefreshMs: 100,
      });
      node.createPublisher("/typed", FakeType);
      node.start();
      node.createSubscription("/cmd", () => {});

      const firstRegisters = requests.filter((request) =>
        request.url.endsWith("/api/v1/topology/register"),
      );
      assert.equal(firstRegisters.length, 2);
      assert.deepEqual(
        firstRegisters.map((request) => [request.body.nodeName, request.body.kind, request.body.topic]),
        [
          ["web_test", "publisher", "/typed"],
          ["web_test", "subscriber", "/cmd"],
        ],
      );
      assert.ok(requests.some((request) =>
        request.url.endsWith("/api/v1/topics/register") &&
        request.body.topic === "/typed" &&
        request.body.typeName === "fake.v1.Msg"
      ));

      await new Promise((resolve) => setTimeout(resolve, 120));
      assert.ok(
        requests.filter((request) => request.url.endsWith("/api/v1/topology/register")).length >= 4,
      );

      node.shutdown();
      assert.equal(
        requests.filter((request) => request.url.endsWith("/api/v1/topology/unregister")).length,
        2,
      );
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("keeps registration failures best-effort", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () => {
      throw new Error("console unavailable");
    }) as typeof fetch;
    try {
      const node = GrpcNode.grpc("web_test", {
        consoleUrl: "http://console.test",
      });
      node.createPublisher("/topic");
      node.start();
      await new Promise((resolve) => setTimeout(resolve, 0));
      node.shutdown();
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
});
