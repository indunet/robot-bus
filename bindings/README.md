# Language bindings

SDK bindings for the Rust core (`Cargo.toml` + `src/` at the repo root).
FFI for Python lives in the core crate (`src/python_api.rs`); Node.js napi
FFI lives under `typescript/native/`. Each binding directory holds the
language-facing package, packaging metadata, and tests.

| Path | Status |
|------|--------|
| [`python/`](python/) | Python binding (maturin / PyO3) |
| [`typescript/`](typescript/) | TypeScript hybrid npm SDK (napi-rs Node + gRPC-Web browser) |
| [`cpp/`](cpp/) | C++ SDK (C ABI + RAII; DEB/MSI via GitHub Releases) |
| [`kotlin/`](kotlin/) | Kotlin JVM JAR (`org.indunet:robot-bus`) |
| [`android/`](android/) | Android AAR (`org.indunet:robot-bus-android`; JNA + jniLibs) |

`console/` is a product UI embedded into the broker, not an SDK binding.
