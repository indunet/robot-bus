English | [中文](../zh/support.md)

# Support and verification status

This page describes the implementation and traceable verification evidence for repository version **2.3.1** (documentation reviewed 2026-09-08). A configured CI check does not mean a particular commit passed. For a release, inspect the matching tag's build results and actual artifacts.

## Core SDKs

Core SDKs need no ROS installation unless the bridge is enabled. The table reflects the [regular CI configuration](../../.github/workflows/ci.yml), not certification of every OS, architecture, or language version.

| Client | Implementation scope | Repository checks | Not established by these checks |
| --- | --- | --- | --- |
| Rust | Native topic/service/action, WS client, broker | Native and feature combination tests, example builds, interop | Runtime behavior on every target platform |
| Python | Native topic/service/action, WS client, broker | Typed, native and interop tests on Python 3.12; execution of README/API quick starts | Execution on every declared Python version |
| TypeScript / Node.js | Native SDK, WS client, broker | npm tests and native interop | Every Node.js/platform combination |
| Browser | WS publish/subscribe and service/action clients | WS protocol and client tests | Complete browser matrix; service/action server registration is unavailable |
| C++ / Java | Native SDK, WS client, broker | Native interface tests and interop | Runtime validation of every release artifact in the user's environment |
| Android | Independent Kotlin SDK, minSdk 24 | JNI and AAR builds, host JVM unit tests | Comprehensive device/emulator, background lifecycle or network recovery testing |

Consult the [language API guides](../../README.md#16-documentation) for interface differences. WS Nodes operate as clients and cannot register service/action servers.

## ROS 2 bridge: targets and evidence

Target distributions are **Humble / Jazzy**. Topic, service and action bridging is implemented in both directions, with separate routes for each direction. Targeting a distribution does not establish live acceptance coverage for every language and feature.

| Language | Target distributions | Topics | Services / actions | Verification boundary |
| --- | --- | --- | --- | --- |
| Rust / rclrs | Humble, Jazzy | Mappers and routes implemented | Both directions implemented | Regular CI type-checks `ros2-shim` with Humble configuration without linking real ROS; the 2026-08-31 host record reports rust IDL type mismatches |
| Python / rclpy | Humble, Jazzy | Mappers and routes implemented | Both directions implemented | Regular CI includes simulated and bus-side tests; live service and action feedback/result/timeout/cancel verification remains pending |
| C++ / rclcpp | Humble, Jazzy | Mappers and routes implemented | Both directions implemented | The [release workflow](../../.github/workflows/release-cpp.yml) configures Linux ROS package builds; packaging is not route-by-route runtime acceptance; live service/action verification remains pending |

See the [bridge test record](ros2-bridge-perf-report.md) for the historical failing environment and commands. It does not prove every current machine fails, or that the current commit has fixed that failure. In addition to sourcing ROS, Rust needs compatible rust IDL and typesupport. Passing shim checks alone does not establish runtime support.

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

The [bus report](perf-report.md) contains historical single-host samples, missing scenarios and anomalous WS counts. It cannot establish a ROS 2 selection ranking. See [performance interpretation and reproduction](performance.md) for usable conclusions, unresolved numbers and the information needed for a new run.
