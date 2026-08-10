# robot-bus console

Broker monitoring console: status, topic / service / action traffic, event logs, and live topology.

The sidebar **TANK** entry opens one **TANK** floating window (also `/tank/`). Opening it acquires a console session that lazy-starts the in-process `tank` inside the broker. Multiple browsers share one world (`cmd_vel` is last-writer-wins).

| Node | Where | Role |
|------|---------|------|
| `tank` | broker (on session acquire) | Sim: SUB `/robot_bus/tank/cmd_vel`, integrate pose, PUB `/robot_bus/tank/pose` |
| `tank_viz` | console TANK / `/tank/` | Viz/ops: session + SUB pose + PUB cmd_vel |

```bash
cargo run --bin robot_bus_broker          # terminal 1
# open console TANK window or /tank  (starts tank automatically)
```

Both Tank pages and the Dashboard use the in-repo TypeScript `WsNode` and connect to
**same-origin** WebSocket RPC at `/ws` (broker default `http://127.0.0.1:15570` → `ws://127.0.0.1:15570/ws`).

Monitoring data is published by the broker on system topics:

- `/robot_bus/status`
- `/robot_bus/topics`
- `/robot_bus/services`
- `/robot_bus/actions`
- `/robot_bus/topology`
- `/robot_bus/events`

A REST shim remains for CLI tooling (`rbus`): `GET /api/v1/status|topics|services|actions|topology` and `SSE /api/v1/events`.

UI copy supports EN / 中文 (default EN; preference in `localStorage` key `robot-bus-console-locale`).

### Topology (read-only)

Sidebar **TOPOLOGY**: pub/sub graph from best-effort control-plane service registration on `Node::create_publisher` / `create_subscription` (`/robot_bus/topology/register` and `/robot_bus/topic_type/register`). Endpoints expire after ~30s without refresh; crashed processes rely on TTL cleanup.

Domain visualizers, Flow plumbing, and LIVE / WHEP live in the sibling **[robot-bus-tools](https://github.com/indunet/robot-bus-tools)** Studio — not in this console.

## Development (recommended)

Start the broker, then hot-reload the frontend. `pnpm dev` proxies `/api/*` to the broker;
browser WebSocket RPC connects to the broker directly (default `ws://127.0.0.1:15570/ws`):

```bash
# repo root
cargo run --bin robot_bus_broker

# other terminal
cd console
pnpm install
pnpm dev
# http://localhost:3020  (UI; WS/REST go to broker :15570 directly)
```

Custom broker URL (Next rewrite + browser client):

```bash
ROBOT_BUS_BROKER_URL=http://127.0.0.1:25770 \
NEXT_PUBLIC_ROBOT_BUS_BROKER_URL=http://127.0.0.1:25770 \
pnpm dev
```

## Embedded in the broker

Static export synced to `assets/console/` (gitignored):

```bash
# repo root
just console
# equivalent: pnpm build && ../scripts/sync_console_assets.sh
cargo run --bin robot_bus_broker
# http://localhost:15570
```

Builds with the `console` feature require `assets/console/index.html` or `build.rs` fails and tells you to run `just console`.
