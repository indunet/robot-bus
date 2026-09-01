/**
 * Standalone broker CLI is `npx robot-bus` (package bin → dist/cli.js).
 *
 * Needs the napi addon: `npm run build:native` (or `just ts-dev`).
 * Skips when the native binary is missing.
 */

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import type { NativeBinding } from "../src/native.js";

async function tryLoadNative(): Promise<NativeBinding | null> {
  try {
    const { loadNative } = await import("../src/native.js");
    return loadNative();
  } catch {
    return null;
  }
}

describe("broker CLI", () => {
  it("prints help via --help", async () => {
    const native = await tryLoadNative();
    if (!native?.runBroker) {
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
