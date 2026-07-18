# robot-bus console

Web 监控控制台：查看 broker 状态、topic 流量与事件日志。

对接 broker 同端口 API：

- `GET /api/v1/status`
- `GET /api/v1/topics`
- `SSE /api/v1/events`

Service / Action 统计尚未接入（表为空）。开发时用 `pnpm dev` 需自行把 `/api` 代理到 broker，或直接打开嵌入后的 `http://localhost:15771`。

## 开发

```bash
pnpm install
pnpm dev
# http://localhost:3000  （无 broker 时代理会失败；联调请用下方嵌入路径）
```

```bash
pnpm build   # 静态导出到 out/（供 broker 嵌入）
```

同步到 Rust 嵌入目录并重新编译 broker：

```bash
# 从仓库根目录：
just console && cargo run --bin robot_bus_broker
# 或在 console/ 下：
pnpm build && ../scripts/sync_console_assets.sh
cd .. && cargo run --bin robot_bus_broker
# http://localhost:15771
```
