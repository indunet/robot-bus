# Language bindings

SDK bindings for the Rust core (`Cargo.toml` + `src/` at the repo root).
PyO3 FFI lives in the core crate (`src/python_api.rs`); each binding directory
holds the language-facing package, packaging metadata, and tests.

| Path | Status |
|------|--------|
| [`python/`](python/) | Python binding (maturin / PyO3) — first binding |
| `cpp/` | Planned |
| `kotlin/` | Planned |
| `typescript/` | Planned (npm SDK; distinct from [`../console/`](../console/) Web UI) |

`console/` is a product UI embedded into the broker, not an SDK binding.
