# robot-bus console

Broker monitoring console: status, topic / service / action traffic, event logs, and live topology.

The sidebar **BOT** entry opens the browser-based micro-robot simulation floating windows (also available as standalone pages):

- `/bot_sim/` — canvas simulation; subscribes `cmd_vel`, publishes `pose`
- `/bot_teleop/` — WASD / arrow-key controller

Both pages use the in-repo TypeScript `GrpcNode` and connect to the broker's
gRPC-Web endpoint reported by `/api/v1/status`.

Talks to the broker on the same port:

- `GET /api/v1/status`
- `GET /api/v1/topics`
- `GET /api/v1/services`
- `GET /api/v1/actions`
- `GET /api/v1/topology` — process nodes ↔ topic edges (best-effort client registration)
- `POST /api/v1/topology/register` / `unregister`
- `SSE /api/v1/events`

UI copy supports EN / 中文 (default EN; preference in `localStorage` key `robot-bus-console-locale`).

### Topology (read-only)

Sidebar **TOPOLOGY**: pub/sub graph from best-effort HTTP registration on `Node::create_publisher` / `create_subscription`. Endpoints expire after ~30s without refresh; crashed processes rely on TTL cleanup. Paths that skip the Rust `Node` (or never reach the console) may not appear.

Domain visualizers, Flow plumbing, and LIVE / WHEP live in the sibling **[robot-bus-tools](https://github.com/indunet/robot-bus-tools)** Studio — not in this console.

## Development (recommended)

Start the broker, then hot-reload the frontend. `pnpm dev` proxies `/api/*` to the broker (default `http://127.0.0.1:15771`):

```bash
# repo root
cargo run --bin robot_bus_broker

# other terminal
cd console
pnpm install
pnpm dev
# http://localhost:3000
```

Custom broker URL:

```bash
ROBOT_BUS_BROKER_URL=http://127.0.0.1:25771 pnpm dev
```

## Embedded in the broker

Static export synced to `assets/console/` (gitignored):

```bash
# repo root
just console
# equivalent: pnpm build && ../scripts/sync_console_assets.sh
cargo run --bin robot_bus_broker
# http://localhost:15771
```

Builds with the `console` feature require `assets/console/index.html` or `build.rs` fails and tells you to run `just console`.
