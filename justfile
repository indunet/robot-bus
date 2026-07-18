# Lightweight task runner — does not replace cargo / maturin / pnpm.
# Install: https://github.com/casey/just

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Generate Python protobuf modules from proto/
proto:
	python3 scripts/generate_python_msgs.py

alias gen-python := proto

# Build and install the Python binding into the active venv
python-dev:
	cd bindings/python && maturin develop --features extension-module,grpc

# Build console and sync static assets into assets/console for rust-embed
console:
	cd console && pnpm build
	./scripts/sync_console_assets.sh

# Rust tests (default features)
test-rust:
	cargo test

# Rust tests without default features
test-rust-minimal:
	cargo test --no-default-features

# Pure-Python message / typed-API smoke tests (no native extension required)
test-python:
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_msgs_roundtrip.py
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_typed_api.py

# Local checks aligned with CI (codegen freshness + rust + python smoke)
ci: proto
	git diff --exit-code -- bindings/python/robot_bus
	just test-rust
	just test-rust-minimal
	just test-python
