# robot-bus console

Broker monitoring console: status, topic / service / action traffic, event logs, and live topology.

The sidebar **BOT** entry opens the browser bot nodes as floating windows (also available as standalone pages). Physics runs in a separate Rust process:

| Node | Process | Role |
|------|---------|------|
| `bot_sim` | `cargo run --example bot_sim` | SUB `/bot1/cmd_vel`, integrate pose, PUB `/bot1/pose` |
| `bot_sim_viewer` | `/bot_sim/` | Canvas viewer — SUB pose only |
| `bot_control_panel` | `/bot_teleop/` | WASD / arrow-key control — PUB cmd_vel |

```bash
cargo run --bin robot_bus_broker          # terminal 1
cargo run --example bot_sim               # terminal 2  (or: just bot-sim)
# open console BOT windows or /bot_sim + /bot_teleop
```

Both Bot pages and the Dashboard use the in-repo TypeScript `GrpcNode` and connect to
**same-origin** gRPC-Web (broker default `http://127.0.0.1:15770`).

Monitoring data is published by the broker on system topics:

- `/_robot_bus/status`
- `/_robot_bus/topics`
- `/_robot_bus/services`
- `/_robot_bus/actions`
- `/_robot_bus/topology`
- `/_robot_bus/events`

A REST shim remains for CLI tooling (`rbus`): `GET /api/v1/status|topics|services|actions|topology` and `SSE /api/v1/events`.

UI copy supports EN / 中文 (default EN; preference in `localStorage` key `robot-bus-console-locale`).

### Topology (read-only)

Sidebar **TOPOLOGY**: pub/sub graph from best-effort control-plane service registration on `Node::create_publisher` / `create_subscription` (`/_robot_bus/topology/register` and `/_robot_bus/topic_type/register`). Endpoints expire after ~30s without refresh; crashed processes rely on TTL cleanup.

Domain visualizers, Flow plumbing, and LIVE / WHEP live in the sibling **[robot-bus-tools](https://github.com/indunet/robot-bus-tools)** Studio — not in this console.

## Development (recommended)

Start the broker, then hot-reload the frontend. `pnpm dev` proxies `/api/*` and gRPC-Web paths to the broker (default `http://127.0.0.1:15770`):

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
ROBOT_BUS_BROKER_URL=http://127.0.0.1:25770 pnpm dev
```

## Embedded in the broker

Static export synced to `assets/console/` (gitignored):

```bash
# repo root
just console
# equivalent: pnpm build && ../scripts/sync_console_assets.sh
cargo run --bin robot_bus_broker
# http://localhost:15770
```

Builds with the `console` feature require `assets/console/index.html` or `build.rs` fails and tells you to run `just console`.
