[English](../en/architecture.md) | 中文

# 架构

Robot Bus 是一个多语言消息框架，采用类似 ROS 2 的节点模型。一个 broker 负责路由话题、服务和 Action，消息载荷使用 Protobuf。连接配置见[部署与排障](deployment.md)，桥接配置见 [ROS 2 桥接](ros2-bridge.md)。

## 系统概览

![系统概览：客户端、broker 的通信功能与控制面](../images/architecture-overview.svg)

*各语言的 SDK 和浏览器共用一个 broker。HTTP 发现接口、控制台和 WebSocket RPC 共用同一个 HTTP 端口。*

## Broker 与传输方式

![Broker 路由、ZeroMQ 传输方式与 WebSocket 的限制](../images/architecture-broker.svg)

*原生节点使用 `inproc`、`ipc` 或 `tcp`。浏览器可以通过 WebSocket 发起调用，但无法注册服务端。*

## 节点模型

![节点编程模型：Context、Node、节点实体与 spin](../images/architecture-node.svg)

*节点先注册发布者、订阅者等实体，再通过 `start()` 或 `spin()` 驱动回调。创建 `Node` 时不会建立连接。*

## Broker 联邦

![两个 broker 先通过 HTTP 发现对端，再经 ZeroMQ 转发话题、服务和 Action](../images/architecture-federation.svg)

*按需启用 broker 联邦。启用后，broker 通过发现接口获取对端信息，再转发话题、服务和 Action。客户端继续连接各自的 broker。*

## ROS 2 桥接

![连接边缘主机与 ROS 2 通信图的桥](../images/architecture-ros2-bridge.svg)

*边缘主机继续使用 Protobuf。进程内的桥连接 Robot Bus 与 ROS 2 Humble 或 Jazzy，每条路由单独配置映射方向。两个系统只在桥接进程内交接。*
