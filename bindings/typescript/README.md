# robot-bus TypeScript SDK

Hybrid npm package:

| Environment | Entry | Capabilities |
|-------------|-------|----------------|
| Node.js | napi-rs native addon | Full ZMQ Node API (publish, servers, broker) |
| Browser | WebSocket RPC client | Subscribe / publish / service call / action (no servers) |

Bundlers pick the right entry via `package.json` `exports` (`browser` vs default).

```bash
npm install robot-bus
# local: just ts-dev
```

See [`docs/en/typescript-api.md`](../../docs/en/typescript-api.md).
