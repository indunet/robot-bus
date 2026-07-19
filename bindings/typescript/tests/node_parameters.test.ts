/**
 * Node local parameters + YAML load.
 *
 * Needs the napi addon: `npm run build:native` (or `just ts-dev`).
 * Skips cleanly when the native binary is missing.
 */

import assert from "node:assert/strict";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import type { NativeBinding } from "../src/native.js";

async function tryLoadNative(): Promise<NativeBinding | null> {
  try {
    const { loadNative } = await import("../src/native.js");
    return loadNative();
  } catch {
    return null;
  }
}

describe("node parameters", () => {
  it("declare/get/set/list and yaml", async () => {
    const native = await tryLoadNative();
    if (!native?.Node) {
      return;
    }

    const node = new native.Node("params");
    node.declareParameter("max_speed", 1.5);
    node.declareParameter("frame_id", "base_link");
    node.declareParameter("enabled", true);
    node.declareParameter("count", 3);

    assert.equal(node.getParameter("max_speed"), 1.5);
    assert.equal(node.getParameter("frame_id"), "base_link");
    assert.equal(node.getParameter("enabled"), true);
    assert.equal(node.getParameter("count"), 3);
    assert.equal(node.hasParameter("frame_id"), true);
    assert.equal(node.hasParameter("missing"), false);

    node.setParameter("max_speed", 2.0);
    assert.equal(node.getParameter("max_speed"), 2.0);

    const listed = node.listParameters();
    assert.equal(listed.length, 4);

    node.loadParametersFromYamlStr(
      "ros__parameters:\n  max_speed: 3.25\n  extra: hello\n",
    );
    assert.equal(node.getParameter("max_speed"), 3.25);
    assert.equal(node.getParameter("extra"), "hello");

    const dir = await mkdtemp(join(tmpdir(), "robot-bus-params-"));
    const path = join(dir, "p.yaml");
    try {
      await writeFile(path, "count: 9\n");
      node.loadParametersFromYamlFile(path);
      assert.equal(node.getParameter("count"), 9);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
