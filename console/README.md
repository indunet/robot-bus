# robot-bus console

Web 监控控制台：查看 broker 状态、topic 流量与事件日志。

对接 broker 同端口 API：

- `GET /api/v1/status`
- `GET /api/v1/topics`
- `GET /api/v1/services`
- `GET /api/v1/actions`
- `SSE /api/v1/events`

UI 文案支持 EN / 中文（默认 EN，偏好存在 `localStorage` 的 `robot-bus-console-locale`）。

## 开发（推荐）

先启动 broker，再热更新前端。`pnpm dev` 会把 `/api/*` 代理到 broker（默认 `http://127.0.0.1:15771`）：

```bash
# 仓库根目录
cargo run --bin robot_bus_broker

# 另一终端
cd console
pnpm install
pnpm dev
# http://localhost:3000
```

自定义 broker 地址：

```bash
ROBOT_BUS_BROKER_URL=http://127.0.0.1:25771 pnpm dev
```

## 嵌入 broker

静态导出并同步到 `assets/console/`（该目录已 gitignore，不入库）：

```bash
# 仓库根目录
just console
# 等价：pnpm build && ../scripts/sync_console_assets.sh
cargo run --bin robot_bus_broker
# http://localhost:15771
```

带 `console` feature 编译前必须已有 `assets/console/index.html`，否则 `build.rs` 会报错并提示运行 `just console`。
