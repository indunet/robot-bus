import { defineConfig } from "tsup";

export default defineConfig([
  {
    entry: { "index.node": "src/index.node.ts" },
    format: ["esm"],
    dts: true,
    sourcemap: true,
    clean: true,
    target: "node18",
    platform: "node",
    external: [/^robot-bus\./, /\.node$/],
  },
  {
    entry: { "index.browser": "src/index.browser.ts" },
    format: ["esm"],
    dts: true,
    sourcemap: true,
    clean: false,
    target: "es2022",
    platform: "browser",
  },
]);
