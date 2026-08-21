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

Missing peers **fail** the run (CI gate). For a local partial run only:

```bash
ROBOT_BUS_INTEROP_ALLOW_SKIP=1 python3 tests/interop/run.py
```

## Prerequisites

```bash
just python-dev    # Python native extension (embeds Web console)
just console       # embedded UI assets (C ABI / TS native enable `console`)
# `just test-interop` then builds Rust / C++ / Java / TypeScript peers
```

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
