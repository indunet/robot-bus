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
pnpm build   # 生产构建
pnpm start   # 运行构建产物
```
