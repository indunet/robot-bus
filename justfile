# Lightweight task runner — does not replace cargo / maturin / pnpm.
# Install: https://github.com/casey/just

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Generate Python protobuf modules from proto/ (gitignored; needed before test/pack)
proto:
	python3 scripts/generate_python_msgs.py

alias gen-python := proto

# Generate TypeScript protobuf + gRPC stubs (gitignored)
gen-typescript:
	python3 scripts/generate_typescript_msgs.py

# Generate C++ protobuf stubs under bindings/cpp/generated/ (gitignored)
gen-cpp:
	python3 scripts/generate_cpp_msgs.py

# Generate Rust prost/tonic stubs under src/msgs/generated + src/grpc/generated (gitignored)
gen-rust:
	python3 scripts/generate_rust_msgs.py

# All language stubs (protoc 35.1)
gen-all: gen-rust proto gen-typescript gen-cpp

# Build and install the Python binding into the active venv
python-dev: gen-python gen-rust
	cd bindings/python && maturin develop --features extension-module,grpc

# Build TypeScript native addon + JS bundle
ts-dev: gen-typescript gen-rust
	cd bindings/typescript && npm install && npm run build:native && npm run build:ts

# Build C++ FFI + msgs library + tests
cpp-dev: gen-cpp gen-rust
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cmake -S bindings/cpp -B bindings/cpp/build -DCMAKE_BUILD_TYPE=Release
	cmake --build bindings/cpp/build -j

# Run C++ binding tests (ephemeral in-process brokers)
test-cpp:
	cmake --build bindings/cpp/build --target cpp_tests
	./bindings/cpp/build/msgs_roundtrip
	./bindings/cpp/build/timer_spin
	./bindings/cpp/build/pub_sub_imu
	./bindings/cpp/build/service_set_bool
	./bindings/cpp/build/action_fibonacci

# Build console and sync static assets into assets/console for rust-embed
console:
	cd console && pnpm build
	./scripts/sync_console_assets.sh

# Rust tests (default features)
test-rust: gen-rust
	cargo test

# Rust tests without default features
test-rust-minimal: gen-rust
	cargo test --no-default-features

# Pure-Python message / typed-API smoke tests (no native extension required)
test-python: gen-python
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_msgs_roundtrip.py
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_typed_api.py

# TypeScript smoke tests (msgs + GrpcNode guards; no broker required)
test-typescript: gen-typescript
	cd bindings/typescript && npm test

# Local checks aligned with CI (codegen then rust + python/ts smoke)
ci: gen-all
	just test-rust
	just test-rust-minimal
	just test-python
	just test-typescript
