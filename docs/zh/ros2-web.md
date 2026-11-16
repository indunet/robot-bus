[English](../en/ros2-web.md) | 中文

# 浏览器 → Robot Bus → 真实 ROS2

[桥接实验台](../../examples/ros2_web/README.md)提供一条可运行的验收链路，首轮范围为
**ROS2 Humble、Python/rclpy、Fast DDS、Chromium**。Broker 和 Python/TypeScript SDK
均从当前源码构建。ROS 节点在独立进程中运行，与桥之间使用真实 DDS 实体通信。

```text
浏览器 TypeScript SDK ←WebSocket→ Robot Bus broker ←TCP→ Python 桥 ←DDS→ ROS 节点
```

## 启动

在仓库根目录执行：

```bash
docker compose -f examples/ros2_web/compose.yaml up --build
```

打开 <http://127.0.0.1:8080>。发送指令后观察 ROS 返回的内容；点击 Reset 调用 ROS
服务；启动任务查看反馈和结果。把项数设为 30 可以留出取消时间。展开异常路径面板，
可验证拒绝、ROS 中止和桥接超时。ROS 状态以 10 Hz 发布。

这是独立示例界面；broker 使用 `ws,console-api` 构建，无需编译内嵌 Next.js 控制台。
Compose 只在回环地址开放 8080、15560 端口。DDS 留在容器内部
（`ROS_LOCALHOST_ONLY=1`，domain 73）。首次构建需要下载依赖并编译 SDK。

## 路由与断言

下表名称均带 `/ros2_web` 前缀。Topic 方向指消息流向，RPC 方向指发起调用的一侧。

| Bus 路由 | ROS 路由 | 方向 / 契约 | 验收断言 |
| --- | --- | --- | --- |
| `/status` | `/status` | ROS → bus，`std_msgs/String` | 浏览器收到递增的 ROS 状态序号 |
| `/command` | `/command` | Bus → ROS，`std_msgs/String` | 唯一的浏览器指令出现在 ROS 状态中 |
| `/reset` | `/reset` | Bus → ROS，`std_srvs/Trigger` | 返回内容正确，ROS 重置次数加一 |
| `/slow` | `/slow` | Bus → ROS，`std_srvs/Trigger` | 返回类型化 `timeout`，后续重置仍能成功 |
| `/fibonacci` | `/fibonacci` | Bus → ROS，`example_interfaces/Fibonacci` | 反馈、精确结果、拒绝、中止、取消 |
| `/deadline` | `/fibonacci` | Bus → ROS，同一 Action | 返回 `timeout`，ROS 确认收到取消 |

示例计算恰好 `order` 项 Fibonacci 数列。项数超出 1–30 时拒绝，13 项用于主动中止演示。
普通 Action 的桥接预算是 10 秒，deadline 路由是 0.4 秒。slow 服务运行 2 秒，桥接预算
为 0.4 秒。**超时不是回滚**，ROS 服务仍会完成。订阅建立是异步的，Topic 测试可在有限
时间内重复同一条幂等指令；RPC 不自动重试。

## 自动验收与证据

[测试入口](../../examples/ros2_web/test.sh)启动后端，用 Chromium 执行八个 Playwright
测试；等待均有上限，缺少依赖不会跳过。[CI 工作流](../../.github/workflows/ros2-web.yml)
在 Ubuntu 22.04 上构建同一 Dockerfile，测试失败也会保存证据。

```bash
docker build -f examples/ros2_web/Dockerfile -t robot-bus-ros2-web:test .
docker run --name ros2-web-acceptance --init \
  -e ROBOT_BUS_TEST_COMMIT="$(git rev-parse HEAD)" \
  robot-bus-ros2-web:test bash test.sh
# 测试失败后也执行复制：
docker cp ros2-web-acceptance:/workspace/examples/ros2_web/test-results ./ros2-web-results
docker rm ros2-web-acceptance
```

结果包含 `environment.log`（提交、系统/架构、ROS/RMW 配置及工具版本）、
`backend.log`、JUnit 报告，以及失败时的浏览器 trace 和截图。配置了工作流不代表特定
提交已经通过，应查看对应运行与附件。Docker 标签和 apt 仓库可能变化，环境日志记录
实际使用的版本。容器保留供排障，再次运行前删除它或换一个名称。

## 已 source Humble 的本机开发

使用 Ubuntu 22.04、ROS 配套的 Python 3.10、Node 22、Rust 和 protoc 35.1。
venv 必须能访问系统 ROS Python 包。在仓库根目录执行：

```bash
source /opt/ros/humble/setup.bash
python3 -m venv --system-site-packages .venv
source .venv/bin/activate
pip install 'maturin>=1.7,<2' 'protobuf>=7.35,<8'
python3 scripts/generate_python_msgs.py
python3 scripts/generate_rust_msgs.py
npm ci --prefix bindings/typescript
python3 scripts/generate_typescript_msgs.py
npm run build:ts --prefix bindings/typescript
(cd bindings/python && maturin develop --no-default-features --features extension-module,ws,console-api)
cd examples/ros2_web
npm ci
npm run build
export ROS_LOCALHOST_ONLY=1 ROS_DOMAIN_ID=73 RMW_IMPLEMENTATION=rmw_fastrtps_cpp
python3 run.py
# 或停止 run.py 后执行自动验收：
npx playwright install --with-deps chromium
bash test.sh
```

缺少 ROS 接口时安装 `ros-humble-example-interfaces`。
可用 `run.py --web-port 8081 --broker-port 15561` 修改演示端口；打开打印的 URL，
其中 `brokerPort` 参数会配置浏览器。自动验收使用默认端口并自行管理后端。
连接失败会显示在界面上；断连后点击 Connect 重新检查路由就绪状态。

## 范围

本验收覆盖一个 ROS 节点与一个 Python 桥，不据此保证 Jazzy、Rust/C++、Android、
ROS 主动调用 service/action、多机 DDS、重启恢复、性能或长期稳定性。
其他范围见[支持状态](support.md)。示例不控制机器人硬件。
