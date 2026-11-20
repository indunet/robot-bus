# Changelog

This changelog records user-visible additions, changes, fixes, and breaking changes. It follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Record changes under the applicable version. Explain what changed, why it changed, and how to migrate when needed.

## [Unreleased]

## [2.4.1] - 2026-11-20

### Added

- Brokers on the same host can now federate over IPC. Use `--peer ipc+HOST:PORT` to fetch the peer's discover data over HTTP and link through its `ipc://` sockets. `--message-peer`, `--service-peer`, and `--action-peer` also accept explicit `ipc://` endpoints. If the peer was started with `--tcp-only`, or its socket files are not visible from this host (for example, from another container), startup fails with an explanation instead of silently failing to connect.
- The new `--ipc-dir DIR` option (`DiscoveryConfig::ipc_dir` in Rust) pins a broker's IPC socket directory. By default, each start creates a new temporary directory. Topic federation requires each broker to peer the other. The broker that starts second therefore needs a fixed directory, so that the first broker can name its sockets in its `*-peer` flags. Use one directory per broker.

### Fixed

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
