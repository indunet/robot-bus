# robot-bus console

Web 监控控制台：查看 broker 状态、topic 流量、事件日志、**实时拓扑**、**管道 Flow 编排**（含 ROS 2 bridge），以及 **LIVE**（WHEP WebRTC 播放）。

对接 broker 同端口 API：

- `GET /api/v1/status`
- `GET /api/v1/topics`
- `GET /api/v1/services`
- `GET /api/v1/actions`
- `GET /api/v1/topology` — 进程节点 ↔ topic 边（由客户端尽力登记）
- `POST /api/v1/topology/register` / `unregister`
- `SSE /api/v1/events`

UI 文案支持 EN / 中文（默认 EN，偏好存在 `localStorage` 的 `robot-bus-console-locale`）。

### Topology（只读）

侧栏 **TOPOLOGY**：根据 `Node::create_publisher` / `create_subscription` 的 best-effort HTTP 登记绘制 pub/sub 图。端点约 30s 无刷新会过期；进程崩溃后依赖 TTL 清理。未走 Rust `Node`（或未连上 console）的路径可能不出现在图中。

### Flow（可编辑管道）

侧栏 **FLOW**：编排配置驱动的管道节点（`rbus_usb_camera` / `image_encoder` / `image_decoder` / `webrtc` / `ros2_bridge`）。拖拽节点库、端口连线即 topic 接线；与 Topology 对照显示 live / missing；不在 flow 中的业务进程以只读 external ghost 出现。

- Import / Export `flow.yaml`（旧版纯 Ros2Bridge YAML 会自动升格为单节点 flow）
- 可导出单节点 `ros__parameters` 或 bridge YAML，以及手动 launch 命令文本
- **不会**自动启停进程或热更新 running 节点 / bridge

### LIVE

侧栏 **LIVE**：填写 `rbus_webrtc` 的 WHEP 地址（默认 `http://127.0.0.1:8090/whep`，存 `localStorage`），连接后播放 H.264/Opus，并显示 DataChannel 日志。不走 gRPC-Web；依赖节点侧 CORS。

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
