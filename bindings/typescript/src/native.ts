/**
 * Load the napi-rs native addon built next to the package root.
 * Prefer platform-specific filenames produced by `@napi-rs/cli`.
 */

import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync } from "node:fs";

const require = createRequire(import.meta.url);
const here = dirname(fileURLToPath(import.meta.url));

function candidates(): string[] {
  const root = join(here, "..");
  const platform = process.platform;
  const arch = process.arch;
  const triples: string[] = [];
  if (platform === "darwin" && arch === "arm64") {
    triples.push("darwin-arm64");
  } else if (platform === "darwin" && arch === "x64") {
    triples.push("darwin-x64");
  } else if (platform === "linux" && arch === "x64") {
    triples.push("linux-x64-gnu");
  } else if (platform === "linux" && arch === "arm64") {
    triples.push("linux-arm64-gnu");
  } else if (platform === "win32" && arch === "x64") {
    triples.push("win32-x64-msvc");
  }

  const names = [
    ...triples.map((t) => `robot-bus.${t}.node`),
    "robot-bus.node",
    "index.node",
  ];
  return names.flatMap((name) => [join(root, name), join(here, name)]);
}

export type NativeBinding = typeof import("./native-types.js");

let cached: NativeBinding | null = null;

export function loadNative(): NativeBinding {
  if (cached) {
    return cached;
  }
  const errors: string[] = [];
  for (const path of candidates()) {
    if (!existsSync(path)) {
      continue;
    }
    try {
      cached = require(path) as NativeBinding;
      return cached;
    } catch (err) {
      errors.push(`${path}: ${err}`);
    }
  }
  throw new Error(
    `Failed to load robot-bus native addon. Tried:\n${candidates().join("\n")}\n` +
      (errors.length ? `\nErrors:\n${errors.join("\n")}` : "") +
      `\nBuild with: cd bindings/typescript && npm run build:native`,
  );
}
