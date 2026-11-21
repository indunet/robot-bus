English | [中文](../zh/support.md)

# Support and verification status

This page records implementation and verification evidence for the **2.3.2** development tree. Consult CI runs and their artifacts for results for specific commits and release packages.

## Core SDKs

Core SDKs need no ROS installation unless the bridge is enabled. The table reflects the [regular CI configuration](../../.github/workflows/ci.yml), not certification of every OS, architecture, or language version.

| Client | Implementation scope | Repository checks | Not established by these checks |
| --- | --- | --- | --- |
| Rust | Native topic/service/action, WS client, broker | Native and feature combination tests, example builds, interop | Runtime behavior on every target platform |
| Python | Native topic/service/action, WS client, broker | Typed, native and interop tests on Python 3.12; execution of README/API quick starts | Execution on every declared Python version |
| TypeScript / Node.js | Native SDK, WS client, broker | npm tests and native interop | Every Node.js/platform combination |
| Browser | WS publish/subscribe and service/action clients | WS protocol and client tests | Complete browser matrix; service/action server registration is unavailable |
| C++ / Java | Native SDK, WS client, broker | Native interface tests and interop | Runtime validation of every release artifact in the user's environment |
| Android | AAR that compiles the shared Java SDK, minSdk 24 | JNI and AAR builds, host JVM unit tests | Comprehensive device/emulator, background lifecycle or network recovery testing |

Consult the [language API guides](../../README.md#16-documentation) for interface differences. WS Nodes operate as clients and cannot register service/action servers.

The [release workflow](../../.github/workflows/release-cpp.yml) job `sdk-smoke` downloads the Linux amd64 and Windows x64 core SDK archives and builds a FetchContent consumer (a protobuf round-trip and an in-process node) on a runner that does not install protobuf or Rust. The same workflow uploads arm64, macOS, and ROS 2 archives; this smoke job covers the two core archives above.

## Reliability regression tests

The Linux Rust CI job runs real TCP and WebSocket traffic through a broker and native workers.

| Scenario | Check |
| --- | --- |
| Broker process killed and restarted on the same ports | The existing WS subscription resumes, and the existing native Service and Action workers accept new requests. An in-flight service call fails on disconnect and is not replayed. |
| Silent interruption of the WS TCP path | The client detects the outage by heartbeat and recovers its subscription and fresh service calls when the path returns. The broker stays running. |
| Slow subscriber | After 128 messages arrive while callbacks are paused, the depth-three and depth-one filters deliver their newest three and one messages. A second client receives every sequence and RPC still works on the paused client's socket. |
| Short soak | After five seconds of warmup, six ten-second windows record process RSS, open Linux file descriptors and topic and service p95 latency. Traffic continues during repeated subscription registration and cleanup. |

The soak samples the broker and clients in one process. Its regression limits allow up to 32 MiB peak RSS growth and six additional open descriptors after warmup. For each latency series, the higher p95 of the last two windows must stay within the larger of three times the first two windows' higher p95 or that p95 plus 20 ms. Change `ROBOT_BUS_SOAK_WINDOW_SECONDS` for a longer run.

```sh
export TOKIO_WORKER_THREADS=2
cargo test --no-default-features --features ws,console-api --test ws_reliability -- --nocapture --test-threads=1
cargo test --no-default-features --features ws,console-api --test ws_reliability -- --ignored --nocapture --test-threads=1
```

The CI artifact `rust-reliability-evidence` contains the fault and soak logs, including per-window JSON records. See the [test suite](../../tests/ws_reliability.rs) and [CI workflow](../../.github/workflows/ci.yml).

## ROS 2 bridge: targets and evidence

Target distributions are **Humble / Jazzy**. Topic, service and action bridging is implemented in both directions, with separate routes for each direction. Targeting a distribution does not establish live acceptance coverage for every language and feature.

| Language | Target distributions | Topics | Services / actions | Verification boundary |
| --- | --- | --- | --- | --- |
| Rust / rclrs | Humble, Jazzy | Mappers and routes implemented | Both directions implemented | Built and linked against Humble and Jazzy on 2026-11-20. Verified on-demand topic forwarding, bidirectional Trigger services, and bidirectional Fibonacci actions |
| Python / rclpy | Humble, Jazzy | Mappers and routes implemented | Both directions implemented | The [browser workflow](ros2-web.md) runs bidirectional topic and browser-initiated RPC tests on Humble and Jazzy. See the distribution-specific artifacts |
| C++ / rclcpp | Humble, Jazzy | Mappers and routes implemented | Both directions implemented | The [release workflow](../../.github/workflows/release-cpp.yml) configures Linux ROS package builds; packaging is not route-by-route runtime acceptance; live service/action verification remains pending |

The Rust build used Ubuntu 22.04.5 and 24.04.5 on x86_64 (`ros:humble-ros-base` and `ros:jazzy-ros-base`), Rust 1.100.0, `rclrs` 0.8.0, `ros-env` 0.3.0, `rosidl_runtime_rs` 0.7.0, and Protobuf compiler 35.1. The bridge library, `ros2_bridge_perf`, and both bridge examples compiled and linked with Rust bindings from each distribution's system packages. See the [bridge guide](ros2-bridge.md) for dependencies and commands and the [historical performance record](ros2-bridge-perf-report.md) for the 2026-08-31 field-type failure.

With Fast DDS and localhost ROS communication, all 16 tests in the `ros2_bridge` integration suite (`ros2_bridge_lazy`, `ros2_bridge_service_smoke`, and `ros2_bridge_diagnostics`) passed on each distribution. The 32 Rust bridge unit tests also passed on each distribution. A temporary copy of the performance harness completed Fibonacci goals in both directions with explicit broker endpoints and reliable ROS QoS. See the [bridge guide](ros2-bridge.md) for the integration test command.

Before using the bridge in an application, run the [bridge examples](../../examples/ros2_bridge/README.md) for your types and directions. Record commit/tag, OS/architecture, ROS distribution, RMW, language, dependency versions, commands and results. Check service success and timeout; for actions also check rejection, feedback, result, cancellation and failure. Attach reproducible evidence when updating verification status.

## Protocol compatibility and upgrades

| Change | Compatibility boundary | Upgrade procedure |
| --- | --- | --- |
| WS V2 → V3 | V2 clients cannot connect to V3 `/ws-rpc` | Coordinate broker and SDK upgrades; do not mix V2/V3 |
| WS KeepLast | New clients use opcode 5, which older V3 brokers may not recognize | Upgrade the broker first, then clients; see [QoS](rust-api.md#high-water-mark-hwm-and-qos) |
| Bridge action errors | Older SDKs cannot classify the new rejection/abort/timeout errors | Upgrade bridge, clients and WS server together; see the bridge guide's call diagnostics |
| Custom Protobuf | The application owns its message contract | Preserve field numbers and do not reuse deleted fields; verify cross-version decoding and business semantics before deployment |

A stable Node programming model does not imply an immutable wire protocol or error taxonomy. Reconnection does not resume service/action calls from before a broker restart. Retries need new request/goal IDs and application handling of duplicate execution.

## Performance evidence

The 2026-11-20 [Robot Bus](perf-report.md) and [ROS 2 Humble](ros2-perf-report.md) reports each include three rounds. ROS 2 was rerun separately on the same host and resource limits after the transport audit isolated SHM and UDPv4. WS receive counts were checked by trial ID and sequence. See [performance interpretation and reproduction](performance.md) for the setup and evidence. [Bridge performance](ros2-bridge-perf-report.md) awaits separate sampling.
