[English](../en/deployment.md) | 中文

# 部署与排障

先完成 [README 的三个上手示例](../../README-zh.md#1-快速开始)，再按部署方式配置连接。支持范围与真实 ROS 验证边界见[支持状态](support.md)。

## 选择进程与传输

| 场景 | 建议 | 要点 |
| --- | --- | --- |
| 一个应用管理总线生命周期 | 内嵌 `RobotBusBroker.start()` | broker 必须活到所有业务节点停止之后；不要在每个节点进程里都启动一个 broker |
| 多进程联调或独立守护进程 | 一个独立 broker，其他进程连接它 | 由服务管理器负责启动、退出和日志；应用等待 broker 就绪 |
| 跨进程/跨机器原生 SDK | TCP + HTTP discover | 默认 ZMQ 数据端口由系统动态分配，不能只开放 HTTP 端口 |
| 同进程通信 | inproc + 共享 `Context` | broker 与节点必须共享同一 Context；不同进程不能用 inproc |
| 浏览器或单一入口的客户端 | WS `/ws-rpc` | 与 HTTP 共用端口；能调用 service/action，不能注册服务端 |

内嵌 broker 的生命周期示例（安装 Python 包后可直接运行；它不发送业务消息）：

```python
import robot_bus

ctx = robot_bus.Context()
with robot_bus.RobotBusBroker.start(context=ctx, api_listen="127.0.0.1:0"):
    node = robot_bus.Node.inproc_with_context(ctx, "application")
    try:
        # 在这里创建 publisher/subscription/service/action，再启动回调循环。
        node.start()
        print("broker and node started")
    finally:
        node.shutdown()
        node.stop()
        node.wait()
# 离开 with 后 broker 停止；实际应用应在 with 内等待业务结束。
```

## 单机启动与检查

```bash
python -m robot_bus.broker --api-listen 127.0.0.1:15560 --tcp-only
```

另一个终端运行：

```bash
curl --fail http://127.0.0.1:15560/api/v1/discover
```

应返回 JSON，其中包含 broker 宣告的连接信息。浏览器访问 `http://127.0.0.1:15560` 应显示控制台。HTTP 正常仅证明 API 可达，原生 TCP 节点还需要连通返回的数据端点。`0.0.0.0` 用来监听所有网卡，不应填成客户端目标地址。

当前限制：启用控制台并把 API 配成 `127.0.0.1:0` 时，实际端口会分配成功，但 discover 中的 `apiUrl` / `consoleUrl` 可能仍带端口 0，导致就绪辅助方法失败。需要 discover/监控的部署请显式配置非零 API 端口；上面的 inproc 生命周期例子不依赖 HTTP 发现。

## 多机与固定端口

以下以 broker 的可达地址 `10.0.0.2` 为例；替换成实际地址。端口是本例显式配置，不是 broker 默认值。

```bash
python -m robot_bus.broker --tcp-only \
  --api-listen 0.0.0.0:15560 --advertise-host 10.0.0.2 \
  --message-xsub-bind tcp://0.0.0.0:15580 \
  --message-xpub-bind tcp://0.0.0.0:15581 \
  --service-frontend-bind tcp://0.0.0.0:15662 \
  --service-backend-bind tcp://0.0.0.0:15663 \
  --action-frontend-bind tcp://0.0.0.0:15664 \
  --action-backend-bind tcp://0.0.0.0:15665 --no-tank
```

| 本例端口 | 用途 | 需要连接的参与者 |
| --- | --- | --- |
| 15560 | HTTP discover、控制台、WS | 所有发现客户端及 WS 客户端 |
| 15580 / 15581 | Topic XSUB / XPUB | 原生发布端 / 订阅端 |
| 15662 / 15663 | Service frontend / backend | 原生调用端 / 服务端 |
| 15664 / 15665 | Action frontend / backend | 原生调用端 / 服务端 |

在客户端机器先检查 discover 返回的地址能否从该机器访问；容器内地址或回环地址不能直接供其他机器连接。NAT/容器映射必须同时匹配宣告地址与端口；仅设置 `--advertise-host` 不会创建端口映射。

Python 原生节点：

```python
import robot_bus
node = robot_bus.Node.discover(
    "remote", transport="tcp", api_url="http://10.0.0.2:15560",
)
```

WS 客户端使用 `robot_bus.Node.ws_at("remote", "http://10.0.0.2:15560")`。HTTPS 页面应使用提供 TLS 终止和 WebSocket upgrade 的 `wss://` 入口。部署时用网络访问控制限制总线与控制接口，按需要在代理处认证；CORS 不是身份认证。`--no-console` 关闭控制台组件，不会关闭 WS/discover，也不能充当访问控制。

## 就绪、回调和退出

- `Node(...)` 构造成功不代表已连接；调用 `wait_for_broker(timeout=5.0)` 或使用显式 `Node.discover(...)`。
- 创建 subscription/server 后必须运行 `spin()`、`spin_once()` 或原生 Node 的 `start()`，否则业务回调不会执行。Python Node 不应移交到另一个 Python 线程；后台执行用 `start()`。
- Service/action 先等待 `wait_for_service` / `wait_for_action_server`，再调用。等待辅助方法依赖 broker 的监控信息，属于尽力检查；精简构建中需注意监控能力是否存在。
- 同一线程阻塞等待结果前，服务端必须已在后台或另一个进程运行。`spin()` 放在 `call()` / `result()` 之后不会解决等待问题。
- 退出时停止节点并等待后台执行结束，再退出 broker 上下文。已经运行的用户回调不会被强制杀死，业务回调应能合作退出。
- broker 重启后可重连，但进行中的 RPC 不会自动恢复。超时不等于操作被撤销；重试有副作用的操作前，先设计幂等键或结果查询。

## 按现象排查

| 现象 | 先检查 | 下一步 |
| --- | --- | --- |
| 15560 被占用 | 是否已经启动另一个 broker | 复用已知 broker，或改 `--api-listen` 并同步客户端 API URL |
| 控制台能打开，原生节点连不上 | discover 中的数据端点、网卡地址、端口映射 | 开放相应 TCP 端口，修正 `--advertise-host` 与固定 bind |
| Topic 无输出 | 发布订阅的 topic/type 是否一致，回调循环是否启动 | 用持续发布验证异步订阅建立；在 Topics/Topology 查看节点与类型 |
| Service/action 等待超时 | 服务端进程及 `start()`/`spin()`、路由名、是否已有 worker | 先跑仓库标准示例；再检查业务回调是否阻塞，避免直接无限重试 |
| inproc 收不到消息 | 是否同一进程、同一 Context | 使用 `Node.inproc_with_context` 与 `RobotBusBroker.start(context=ctx)` |
| WS 断连或 unknown opcode 5 | broker/SDK 版本是否匹配 | 先升级 broker；V2/V3 不可混用；检查代理 WebSocket upgrade |
| 慢订阅、丢消息 | Topics 中待发送队列与 dropped，回调耗时 | 按负载调队列；WS KeepLast 只控制服务端待发送队列，原生 HWM 不保证丢旧留新 |
| ROS 桥不可用或编译失败 | source、rclpy/rclcpp、Rust IDL 与 typesupport | 查[支持状态](support.md)与[桥接常见问题](ros2-bridge.md#常见问题)，保留完整环境和错误 |

提交问题时附版本/commit、系统和架构、语言、传输方式、broker 与节点命令、预期与实际输出；ROS 问题另附发行版、RMW 和 source/overlay 情况。移除日志中的凭据。完整构建选项见[按需构建](build-profiles.md)，队列与错误语义见 [Rust API](rust-api.md)。
