English | [中文](../zh/architecture.md)

# Architecture

Robot Bus is a multi-language messaging framework with a ROS 2–style Node model. One broker routes topics, services, and actions. Payloads are protobuf. These figures are the pictures later docs can cite. Connection setup stays in [deployment](deployment.md). Bridge setup stays in [ROS 2 bridge](ros2-bridge.md).

## System overview

![System overview: clients, broker primitives, and the control plane](../images/architecture-overview.svg)

*Language SDKs and the browser share one broker. Discover, the console, and WebSocket RPC share the HTTP port.*

## Broker and transports

![Broker routing, ZeroMQ transports, and WebSocket limits](../images/architecture-broker.svg)

*Native nodes use inproc, ipc, or tcp. Browsers call in over WebSocket and cannot register servers.*

## Node model

![Node programming model: Context, Node, entities, and spin](../images/architecture-node.svg)

*A node registers entities, then start() or spin() drives callbacks. Constructing a Node does not connect.*

## Broker federation

![Two brokers linked by discover, then ZeroMQ, for topics, services, and actions](../images/architecture-federation.svg)

*Federation is optional. A broker learns its peer from discover, then forwards topics, services, and actions. Clients stay on their own broker.*

## ROS 2 bridge

![ROS 2 bridge between edge hosts and a ROS 2 graph](../images/architecture-ros2-bridge.svg)

*Edge hosts stay on protobuf. An in-process bridge maps one direction onto a Humble or Jazzy graph, and that process is the only meeting point.*
