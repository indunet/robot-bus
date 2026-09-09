import type { TestContext } from "node:test";
import type { NativeBinding } from "../src/native.js";

export const NATIVE_SKIP_REASON =
  "napi addon not built (npm run build:native)";

export async function tryLoadNative(): Promise<NativeBinding | null> {
  try {
    const { loadNative } = await import("../src/native.js");
    return loadNative();
  } catch {
    return null;
  }
}

/** Load the napi addon, or skip (smoke CI). Set ROBOT_BUS_REQUIRE_NATIVE=1 to fail instead. */
export async function loadNativeOrSkip(
  t: TestContext,
  keys: Array<keyof NativeBinding> = [],
): Promise<NativeBinding | null> {
  const native = await tryLoadNative();
  const ok = native != null && keys.every((key) => native[key] != null);
  if (ok) {
    return native;
  }
  if (process.env.ROBOT_BUS_REQUIRE_NATIVE === "1") {
    const need = keys.length ? ` (missing ${keys.join(", ")})` : "";
    throw new Error(
      `napi addon required (ROBOT_BUS_REQUIRE_NATIVE=1) but failed to load${need}`,
    );
  }
  t.skip(NATIVE_SKIP_REASON);
  return null;
}
