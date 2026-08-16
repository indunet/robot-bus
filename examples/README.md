# robot-bus examples

Runnable demos for the three core patterns. Start a broker first, then run a
listener/server and a talker/client in separate terminals.

```bash
# Terminal 1
robot-bus-broker
# or: cargo run --bin robot_bus_broker
```

Shared names:

| Kind | Name |
|------|------|
| Topic | `/examples/imu` |
| Service | `/examples/set_bool` |
| Action | `/examples/fibonacci` |

## Scenarios

### 1. Topic (`topic_imu`)

```bash
# Terminal 2 — listener
python3 examples/topic_imu/python/listener.py
# cargo run --example topic_imu_listener
# ./examples/build/topic_imu_listener   # after `just examples-cpp`

# Terminal 3 — talker
python3 examples/topic_imu/python/talker.py
# cargo run --example topic_imu_talker
# ./examples/build/topic_imu_talker
```

### 2. Service (`service_set_bool`)

```bash
# Terminal 2 — server
python3 examples/service_set_bool/python/server.py

# Terminal 3 — client
python3 examples/service_set_bool/python/client.py
```

### 3. Action (`action_fibonacci`)

Uses `example_interfaces/action/Fibonacci` (ROS-aligned).

```bash
# Terminal 2 — server
python3 examples/action_fibonacci/python/server.py

# Terminal 3 — client
python3 examples/action_fibonacci/python/client.py
```

## Languages

| | Rust | Python | C++ |
|--|------|--------|-----|
| Build | `cargo build --examples` | `just python-dev` (or `pip install robot-bus`) | `just examples-cpp` |
| Run | `cargo run --example <name>` | `python3 examples/.../*.py` | binaries under `examples/build/` |

See also: [docs/en/python-api.md](../docs/en/python-api.md), [rust-api.md](../docs/en/rust-api.md), [cpp-api.md](../docs/en/cpp-api.md).
