# Changelog

This changelog records user-visible additions, changes, fixes, and breaking changes. It follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Record changes under the applicable version. Explain what changed, why it changed, and how to migrate when needed.

## [Unreleased]

## [3.0.0] - 2026-11-23

### Breaking

- The Rust crate is now a workspace. `robot-bus` stays the one crate users install: it holds the client SDK (Node, executors, topic / service / action, discovery) and the broker, WebSocket server, console, tank demo, and `rbus` CLI. `robot-bus-msgs` holds the generated messages and is re-exported by `robot-bus`. `robot-bus-ros2` is the ROS 2 bridge and is not published.
- `robot_bus::ros2_bridge` moved to the separate `robot-bus-ros2` crate. Programs that use the bridge depend on `robot-bus-ros2`. Other Rust code keeps its imports, including `robot_bus::RobotBusBroker`.
- `robot-bus` has a new `broker` feature for the broker, federation, and the `robot_bus_broker` / `rbus` binaries. `ws`, `console-api`, and `demo-tank` imply it. The default set `full` is unchanged (`broker`, `ws`, `console`, `demo-tank`). Client-only programs can use `default-features = false` to leave out the broker and its dependencies (`clap`, `axum`, `rust-embed`).
- Low-level modules (`message_bus`, `service_bus`, `action_bus`, `transports`, `wire`) stay public but are `#[doc(hidden)]`.

### Changed

- Splitting the crates does not change the Protobuf runtime range for Python, C++, Java, or Android.
- The Android AAR (`org.indunet:robot-bus-android`) now compiles the Java SDK sources instead of keeping a second Kotlin copy. `RobotBusAndroid.init` is unchanged: call it before `Node` or `Broker`, and an in-process broker still puts IPC sockets under the app cache directory. Do not depend on `org.indunet:robot-bus` and `org.indunet:robot-bus-android` in the same app; the classes would be duplicated.
- Kotlin code on Android can no longer pass named arguments to `BrokerOptions`, `NodeOptions`, or `DiscoverOpts`, and cannot use named arguments on the Java methods (for example `createPublisher(topic, qosDepth = 5)`). Use `brokerOptions`, `nodeOptions`, and `discoverOpts` when you want default arguments. Pass other arguments by position, or pick an overload.

## [2.4.3] - 2026-11-21

### Fixed

- The prebuilt C++ SDK includes the libraries and tools referenced by Protobuf's CMake package, so a standalone CMake project can configure and link after fetching the archive.
- Linux ROS 2 Humble and Jazzy SDKs link the bridge to the bundled robot-bus library by name and use the system ROS 2 libraries. FetchContent supports CMake 3.22, the version shipped with Ubuntu 22.04.
- The release workflow builds and runs a FetchContent consumer against each Linux core and ROS 2 SDK before uploading it.

## [2.4.2] - 2026-11-21

### Added

- C++ releases now include a prebuilt SDK archive alongside the existing installers. CMake `FetchContent` downloads `robot-bus-sdk_<version>_<platform>_<arch>.tar.gz` (Windows: `.zip`) and links it from the build directory. The archive bundles Protobuf 35.1 and, on Linux, libzmq. Linux ROS 2 Humble and Jazzy archives are published separately. `FetchRobotBus.cmake` selects the archive for the current platform. See the C++ API guide.

## [2.4.1] - 2026-11-20

### Added

- Brokers on the same host can now federate over IPC. Use `--peer ipc+HOST:PORT` to fetch the peer's discover data over HTTP and link through its `ipc://` sockets. `--message-peer`, `--service-peer`, and `--action-peer` also accept explicit `ipc://` endpoints. If the peer was started with `--tcp-only`, or its socket files are not visible from this host (for example, from another container), startup fails with an explanation instead of silently failing to connect.
- The new `--ipc-dir DIR` option (`DiscoveryConfig::ipc_dir` in Rust) pins a broker's IPC socket directory. By default, each start creates a new temporary directory. Topic federation requires each broker to peer the other. The broker that starts second therefore needs a fixed directory, so that the first broker can name its sockets in its `*-peer` flags. Use one directory per broker.

### Fixed

- WebSocket service calls and action goals no longer take about 44 ms each. Neither the broker's HTTP/WebSocket server nor the Rust WebSocket client set `TCP_NODELAY`, so each small frame waited on Nagle's algorithm and the peer's delayed ACK. Both ends now disable Nagle; the console HTTP server does too. On the single-host benchmark, the median WebSocket service call dropped from about 44 ms to 0.15 ms and the action from about 44 ms to 0.32 ms. The performance reports now carry these figures.
- A Rust WebSocket node now dispatches a subscribed message as soon as it arrives. `spin` used to sleep 5 ms between checks of the inbox, which added up to 5 ms (about 4.5 ms median) to every delivery. It now blocks until a message arrives, and `ShutdownHandle::shutdown` interrupts the wait, so stopping no longer waits out the poll timeout. Median WebSocket Subscribe latency dropped from about 4.6 ms to 0.2 ms.
- Known trade-off: the highest 64 B WebSocket Subscribe rate that is delivered without loss fell from about 233,000 to about 132,000 messages/s. With Nagle off, the server's per-frame write and flush sends each frame as its own TCP segment. A control build with only `TCP_NODELAY` shows the same ceiling. Batching frames per flush on the server would recover it and is not done yet.
- Creating a publisher, subscription, service, or action no longer waits on the console topology RPC. With the console disabled, or when the endpoint is `inproc` and the client cannot share the broker context, that call used to block until its 2 second timeout. Register and unregister now run in the background on the node's own ZeroMQ context. The native TypeScript in-process tests were hitting their own deadlines because of this wait.

## [2.4.0] - 2026-11-20

### Breaking Changes

- In Rust, `TypedServiceMapper::error_response` now returns `Option<Response>` and defaults to `None`, which suppresses the reply. Wrap the return value of an existing custom implementation in `Some(...)`. The built-in `Trigger` and `SetBool` mappers have been updated.
- In C++, the mapper base class's `error_response` method now returns `std::optional<Response>` and defaults to `std::nullopt`, which suppresses the reply. Derived mapper implementations can still return a `Response` directly.
- Rust, C++, and Python now handle failed ROS-to-bus service calls (`.from_ros → .to_bus`) the same way. Previously, a custom service without `error_response` could return default values such as `sum=0`, which made a failed call look successful. The bridge now sends the mapper's error response when provided. The built-in `Trigger` and `SetBool` mappers return `success=False` and include the reason. Without an error response, the ROS client times out. If a client expects a reply on failure, handle the timeout or implement `error_response` in the mapper.

### Added

- Python's `Ros2Bridge` now provides `run()`, which blocks, handles Ctrl+C and SIGTERM, and always cleans up on exit. It also provides a repeatable `shutdown()` that closes the internal bus node, a `close()` alias, and support for `with bridge:`. The examples and documentation now use `bridge.run()` in place of `spin()` with manual cleanup.
- Python bridge routes now accept either a mapper object or a type name, as in `.mapper("std_srvs/srv/Trigger")`. The bridge can also infer a type name from the ROS interface name.
- Python's `register_mapper(type_name, mapper)`, available from `robot_bus.ros2_bridge`, lets you register a custom mapper under a type name and reference it by string. Call it directly or use it as a decorator. Misspelled type names prompt suggestions for similar names.
- When a ROS-to-bus service call fails, the bridge logs a rate-limited message with the service name and failure reason. It logs `WARN` when it sends an `error_response` and `ERROR` when it leaves the call unanswered. The `ERROR` message suggests implementing `error_response`. The console failure count increments in both cases.
- Added the C++ `ros2_bridge_service_failure` integration test to verify service failure behavior against `rclcpp`. It builds when tests and `-DROBOT_BUS_ROS2=ON` are enabled.

### Fixed

- Python no longer prints a traceback for `ROS executor failed` when Ctrl+C is pressed or the ROS context shuts down.
- Python's `spin_once` now handles an empty route table without raising `nothing registered` and sleeps when idle to avoid busy-waiting.
- Fixed a crash in `examples/ros2_bridge/python/custom_add_two_ints.py` when spinning with no routes registered.
- Generic RPC warnings could consume the rate limit and suppress service-specific failure logs. In Rust, C++, and Python, `rpc_finish` now accepts a `log` flag so the bridge can suppress the generic warning when it logs the failure itself.

### Changed

- The Rust ROS 2 bridge now uses `rclrs` 0.8, `ros-env` 0.3, and `rosidl_runtime_rs` 0.7. On Jazzy, the `Range` mapper uses the ROS default for variance, since the bus message has no variance field.

### Testing

- Python tests cover the bridge lifecycle, context manager, `run()` exit, executor errors, idle spinning, type names, string mapper references, the mapper registry, unanswered service failures, `error_response` replies, and log rate limiting. The tests ran against ROS 2 in a Jazzy container.
- Two new Rust cases in `tests/ros2_bridge_service_smoke.rs` cover service failures with and without `error_response`. The tests ran against ROS 2 in a Humble container and confirmed that the old behavior, which returned `sum=0`, fails the regression check.
- The C++ `ros2_bridge_service_failure` test ran against ROS 2 in a Humble container and confirmed that the old behavior, which returned `sum=0`, fails the regression check.

### Known Limitations

- Rust service callbacks return a response through `rclrs`. To leave a call unanswered, the bridge keeps its response future pending until shutdown. Each unanswered call leaves one pending future. Implement `error_response` for services that fail frequently.
