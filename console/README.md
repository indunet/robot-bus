# robot-bus console

Web 监控控制台：查看 broker 状态、topic 流量、service / action 与事件日志。

当前为 UI 原型（[`lib/mock-data.ts`](lib/mock-data.ts)），尚未对接真实 broker API。不随 crates.io / PyPI 发布。

## 开发

```bash
pnpm install
pnpm dev
# http://localhost:3000
```

```bash
pnpm build   # 静态导出到 out/（供 broker 嵌入）
```

同步到 Rust 嵌入目录并重新编译 broker：

```bash
../scripts/sync_console_assets.sh
cd .. && cargo run --bin robot_bus_broker
# http://localhost:15771
```
