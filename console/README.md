# robot-bus console

Broker monitoring console: status, topic / service / action traffic, event logs, and live topology.

The sidebar **BOT** entry opens one **BOT SIM** floating window (also `/bot_sim/`). Opening it acquires a console session that lazy-starts the in-process `bot_sim` inside the broker. Multiple browsers share one world (`cmd_vel` is last-writer-wins).

| Node | Where | Role |
|------|---------|------|
| `bot_sim` | broker (on session acquire) | Sim: SUB `/bot1/cmd_vel`, integrate pose, PUB `/bot1/pose` |
| `bot_viz` | console BOT SIM / `/bot_sim/` | Viz/ops: session + SUB pose + PUB cmd_vel |

```bash
cargo run --bin robot_bus_broker          # terminal 1
# open console BOT SIM window or /bot_sim  (starts bot_sim automatically)
```

Both Bot pages and the Dashboard use the in-repo TypeScript `GrpcNode` and connect to
**same-origin** gRPC-Web (broker default `http://127.0.0.1:15770`).

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

Start the broker, then hot-reload the frontend. `pnpm dev` proxies `/api/*` and gRPC-Web paths to the broker (default `http://127.0.0.1:15770`):

```bash
# repo root
cargo run --bin robot_bus_broker

# other terminal
cd console
pnpm install
pnpm dev
# http://localhost:3020  (UI; gRPC/REST go to broker :15770 directly)
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
# http://localhost:15770
```

Builds with the `console` feature require `assets/console/index.html` or `build.rs` fails and tells you to run `just console`.
