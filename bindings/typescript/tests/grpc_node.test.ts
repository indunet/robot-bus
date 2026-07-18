import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { GrpcNode } from "../src/grpc-node.js";

describe("GrpcNode capability guards", () => {
  it("rejects publish / servers", () => {
    const node = GrpcNode.grpc("test");
    assert.throws(() => node.createPublisher("/t"), /not available/);
    assert.throws(() => node.createService("/s", () => new Uint8Array()), /not available/);
    assert.throws(() => node.createActionServer("/a", () => []), /not available/);
  });

  it("exposes grpc factory urls", () => {
    assert.equal(GrpcNode.grpc("a").url, "http://127.0.0.1:15770");
    assert.equal(GrpcNode.grpcAt("a", "http://example:15770/").url, "http://example:15770");
  });
});
