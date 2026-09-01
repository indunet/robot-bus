# robot-bus examples

Runnable demos for the core patterns and the ROS 2 bridge. These examples are
multi-process, so start a **standalone** broker first, then run a
listener/server and a talker/client (or the bridge) in separate terminals.

Application code should prefer embedding the broker in-process
(`RobotBusBroker.start()` / language equivalents) rather than depending on
this CLI.

```bash
# Terminal 1
python -m robot_bus.broker
# or: npx robot-bus
# or: cargo run --bin robot_bus_broker
# C++ package: robot_bus_broker
```

Shared names:

| Kind | Name |
|------|------|
| Topic | `/examples/imu` |
| Service | `/examples/set_bool` |
| Action | `/examples/fibonacci` |
| Bridge (builtin) topic | `/examples/chatter` |
| Bridge (builtin) service | `/examples/reset` |
| Bridge (custom) service | `/examples/add_two_ints` |

## Build / run by language

| | Rust | Python | C++ |
|--|------|--------|-----|
| Build | `cargo build --examples` | `just python-dev` (or `pip install robot-bus`) | `just examples-cpp` |
| Run | `cargo run --example <name>` | `python3 examples/.../*.py` | binaries under `examples/build/` |
| ROS 2 bridge | `--features ros2` | source ROS + `rclpy` | `just examples-cpp-ros2` |

C++ binaries after `just examples-cpp` (ROS bridge binaries need `just examples-cpp-ros2`):

| Scenario | Binaries |
|----------|----------|
| Topic | `topic_imu_listener`, `topic_imu_talker` |
| Service | `service_set_bool_server`, `service_set_bool_client` |
| Action | `action_fibonacci_server`, `action_fibonacci_client` |
| ROS 2 bridge | `ros2_bridge_builtin`, `ros2_bridge_custom_add_two_ints` |

## Scenarios

### 1. Topic (`topic_imu`)

```bash
# Terminal 2 — listener
./examples/build/topic_imu_listener
# python3 examples/topic_imu/python/listener.py
# cargo run --example topic_imu_listener

# Terminal 3 — talker
./examples/build/topic_imu_talker
# python3 examples/topic_imu/python/talker.py
# cargo run --example topic_imu_talker
```

### 2. Service (`service_set_bool`)

```bash
# Terminal 2 — server
./examples/build/service_set_bool_server
# python3 examples/service_set_bool/python/server.py
# cargo run --example service_set_bool_server

# Terminal 3 — client
./examples/build/service_set_bool_client
# python3 examples/service_set_bool/python/client.py
# cargo run --example service_set_bool_client
```

### 3. Action (`action_fibonacci`)

Uses `example_interfaces/action/Fibonacci` (ROS-aligned).

```bash
# Terminal 2 — server
./examples/build/action_fibonacci_server
# python3 examples/action_fibonacci/python/server.py
# cargo run --example action_fibonacci_server

# Terminal 3 — client
./examples/build/action_fibonacci_client
# python3 examples/action_fibonacci/python/client.py
# cargo run --example action_fibonacci_client
```

### 4. ROS 2 bridge (`ros2_bridge`)

Source ROS first (`source /opt/ros/humble/setup.bash` or jazzy). See also
[`examples/ros2_bridge/README.md`](ros2_bridge/README.md).

A **custom** bridge needs both sides in-tree:

| Side | Path |
|------|------|
| ROS `.srv` / `.msg` / `.action` | [`ros2_bridge/ros2/my_pkg/`](ros2_bridge/ros2/my_pkg/) |
| Bus `.proto` | [`ros2_bridge/proto/my_pkg/`](ros2_bridge/proto/my_pkg/) |
| Mapper + mount | `*/custom_add_two_ints.*` |

| Demo | What it shows |
|------|----------------|
| **builtin** | Phase-1 mappers: String / Trigger / Fibonacci |
| **custom_add_two_ints** | Your ROS `.srv` + your bus `.proto` + mapper |

#### Built-in mappers

```bash
# Terminal 2 — pick one language
python3 examples/ros2_bridge/python/builtin.py
# cargo run --example ros2_bridge_builtin --features ros2
# ./examples/build/ros2_bridge_builtin

# Terminal 3 — exercise from ROS (bus servers must exist for service/action)
ros2 topic pub /examples/chatter std_msgs/msg/String "{data: hello}"
```

#### Custom mapper (ROS `.srv` + bus `.proto`)

Definitions:

- ROS: [`ros2/my_pkg/srv/AddTwoInts.srv`](ros2_bridge/ros2/my_pkg/srv/AddTwoInts.srv)
- Bus: [`proto/my_pkg/srv/v1/add_two_ints.proto`](ros2_bridge/proto/my_pkg/srv/v1/add_two_ints.proto)

```bash
# Terminal 2
python3 examples/ros2_bridge/python/custom_add_two_ints.py
# cargo run --example ros2_bridge_custom_add_two_ints --features ros2
# ./examples/build/ros2_bridge_custom_add_two_ints

# Terminal 3 (smoke uses system example_interfaces; same fields as my_pkg .srv)
ros2 service call /examples/add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 40}"
# → sum: 42
```

Sources:

| | Built-in | Custom AddTwoInts |
|--|----------|-------------------|
| Python | [`python/builtin.py`](ros2_bridge/python/builtin.py) | [`python/custom_add_two_ints.py`](ros2_bridge/python/custom_add_two_ints.py) |
| Rust | [`rust/builtin.rs`](ros2_bridge/rust/builtin.rs) | [`rust/custom_add_two_ints.rs`](ros2_bridge/rust/custom_add_two_ints.rs) |
| C++ | [`cpp/builtin.cpp`](ros2_bridge/cpp/builtin.cpp) | [`cpp/custom_add_two_ints.cpp`](ros2_bridge/cpp/custom_add_two_ints.cpp) |

Custom topic / action: same dual files (`.msg`/`.action` ↔ `.proto`), then
`TypedTopicMapper` / `TypedActionMapper`. Full contract:
[docs/en/ros2-bridge.md](../docs/en/ros2-bridge.md) ·
[docs/zh/ros2-bridge.md](../docs/zh/ros2-bridge.md).

## Notes

Bus demos under `examples/<scenario>/cpp/` use `robot_bus/typed.hpp`.
The ROS bridge demos use `<robot_bus/ros2_bridge.hpp>` and link
`robot_bus_ros2_bridge` (`ROBOT_BUS_HAS_ROS2`).

API reference: [cpp-api.md](../docs/en/cpp-api.md),
[python-api.md](../docs/en/python-api.md),
[rust-api.md](../docs/en/rust-api.md).
