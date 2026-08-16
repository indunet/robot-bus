# Language bindings

SDK wrappers around the Rust core at the **repo root** (`Cargo.toml` + `src/`).

Keep this grouping: do not promote `python/`, `java/`, … to top-level peers of a
moved `rust/` tree. Rust is the source of truth; each subdirectory here is a
language-facing package (metadata, stubs, tests, packaging).

| Path | Status |
|------|--------|
| [`python/`](python/) | Python binding (maturin / PyO3; FFI in core `src/python_api.rs`) |
| [`typescript/`](typescript/) | TypeScript hybrid npm SDK (napi-rs Node + WebSocket RPC browser) |
| [`cpp/`](cpp/) | C++ SDK (C ABI `robot_bus_c` + CMake; DEB/MSI/PKG via GitHub Releases) |
| [`java/`](java/) | Java JVM JAR via Maven (`org.indunet:robot-bus`, Java 11+) |
| [`android/`](android/) | Standalone Android Kotlin AAR (`org.indunet:robot-bus-android`; no Java JAR dep) |

`console/` is a product UI embedded into the broker, not an SDK binding.
Cross-language interop lives under `tests/interop/` (`just test-interop`).
Runnable demos: [`../examples/`](../examples/).
