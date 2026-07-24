# Cross-language interop tests

Six scenarios on a shared **TCP** broker, each with a different language pair
(inproc is not used — contexts cannot be shared across processes/languages).

| # | Pair | Pattern |
|---|------|---------|
| 1 | Rust → Python | pub-sub (`Imu`) |
| 2 | Python → Java | pub-sub (`Imu`) |
| 3 | TypeScript → Python | pub-sub (`Imu`) |
| 4 | C++ → Python | service (`SetBool`) |
| 5 | Java → Rust | service (`SetBool`) |
| 6 | Python → C++ | action (`Fibonacci`) |

## Prerequisites

```bash
just python-dev    # Python native extension
just ts-dev        # TypeScript native + dist (scenarios 3)
just java-dev      # Java + librobot_bus_c (scenarios 2, 5)
just cpp-dev       # C++ tests/build (scenarios 4, 6)
```

Missing peers are **skipped** (exit 0 for that scenario); real failures exit non-zero.

## Run

```bash
just test-interop
```

## Layout

| Path | Role |
|------|------|
| `run.py` | Orchestrator (Python broker + peers) |
| `ts_pub.mjs` | TypeScript pub peer |
| `src/bin/robot_bus_interop.rs` | Rust peer (`pub`, `svc-client`) |
| `bindings/cpp/tests/interop_peer.cpp` | C++ peer (`svc-server`, `act-client`) |
| `bindings/java/.../interop/InteropPeer.java` | Java peer (`sub`, `svc-server`) |
