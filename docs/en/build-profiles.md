# Build profiles

WebSocket is a core communication capability. The `ws` feature only provides build-time transport selection; the console and demos are separate components.

Cargo features are additive. Use `--no-default-features` to select a smaller build; adding features alone does not remove the default bundle.

| Build | Cargo selection | Includes |
| --- | --- | --- |
| Native ZMQ SDK | `--no-default-features` | Native ZMQ SDK and broker primitives |
| WebSocket communication | `--no-default-features --features ws` | Native ZMQ + WS clients/server, discovery and subscription queue API |
| Monitoring API | `--no-default-features --features ws,console-api` | WebSocket communication + monitoring REST API and bus control plane |
| Embedded console | `--no-default-features --features ws,console` | Monitoring + embedded web UI, without tank simulation |
| Full (default) | no feature flags, or `--features full` | WebSocket communication + console + tank demo |

`console-api` also works without `ws`, serving HTTP monitoring on `ConsoleBrokerConfig.listen`. `console` enables `console-api` and embeds `assets/console`; `demo-tank` independently includes Rust tank simulation code. Without `demo-tank`, the UI hides the tank entry and session acquisition returns 403, even if the runtime flag is true. Shared console assets still contain the tank view code when `console` is enabled.

From a source checkout, run `just gen-rust` once to generate protobuf stubs. Only builds with `console` require `just console` (Node/npm/pnpm and TypeScript code generation). Native ZMQ, WS, and monitoring builds need no web assets or frontend build tools. Native ZeroMQ build requirements still apply. Published Rust packages include generated stubs.

Convenience commands: `just build-sdk`, `just build-ws`, `just build-monitoring`, and `just build-full`. These produce release builds; the SDK command builds the library, and the others build `robot_bus_broker`.

For a small Rust dependency:

```toml
robot-bus = { version = "2.3.2", default-features = false }
```

Existing default Cargo builds and official Python, Node, C++/Java/Android distributions retain the full bundle. Existing explicit `--no-default-features --features ws,console` builds must add `demo-tank` if they need the demo. Runtime `--no-console` and `--no-tank` flags disable behavior; compile-time features remove components from the build.
