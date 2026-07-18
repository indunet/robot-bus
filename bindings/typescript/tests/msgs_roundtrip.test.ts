import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { String$ } from "../generated/std_msgs/msg/v1/primitives.js";
import { encode, decode } from "../src/typed.js";

describe("typed encode/decode", () => {
  it("round-trips std_msgs String$", () => {
    const msg = String$.create({ data: "hello robot-bus" });
    const bytes = encode(String$, msg);
    const back = decode(String$, bytes);
    assert.ok(back);
    assert.equal(back.data, "hello robot-bus");
  });
});
