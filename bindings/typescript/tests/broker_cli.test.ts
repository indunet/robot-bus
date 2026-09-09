/**
 * Standalone broker CLI is `npx robot-bus` (package bin → dist/cli.js).
 *
 * Needs the napi addon: `npm run build:native` (or `just ts-dev`).
 * Skips when the native binary is missing. Set ROBOT_BUS_REQUIRE_NATIVE=1 to fail.
 */

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { loadNativeOrSkip } from "./load-native.js";

describe("broker CLI", () => {
  it("prints help via --help", async (t) => {
    const native = await loadNativeOrSkip(t, ["runBroker"]);
    if (!native) {
      return;
    }

    const cli = join(dirname(fileURLToPath(import.meta.url)), "../src/cli.ts");
    const proc = spawnSync(
      process.execPath,
      ["--import", "tsx", cli, "--help"],
      { encoding: "utf8" },
    );
    const out = `${proc.stdout ?? ""}${proc.stderr ?? ""}`;
    assert.equal(proc.status, 0, out);
    assert.match(out, /npx robot-bus/);
  });
});
