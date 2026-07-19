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

# Generate Java protobuf stubs under bindings/java/generated/ (gitignored)
gen-java:
	python3 scripts/generate_java_msgs.py

# Generate Rust prost/tonic stubs under src/generated/<pkg>/…/v1/<stem>.rs (gitignored)
gen-rust:
	python3 scripts/generate_rust_msgs.py

# All language stubs (protoc 35.1)
gen-all: gen-rust proto gen-typescript gen-cpp gen-java

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

# Build Java JVM binding with Maven (needs librobot_bus_c from cpp native)
java-dev: gen-java
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cd bindings/java && mvn test

# Install Java SDK into ~/.m2 (required before android-dev)
java-install: gen-java
	cd bindings/java && mvn -DskipTests install

# Cross-compile librobot_bus_c for Android ABIs into bindings/android jniLibs
android-native:
	./scripts/build_android_native.sh

# Assemble Android AAR (needs java-install + android-native)
android-dev: java-install android-native
	cd bindings/android && ./gradlew assembleRelease --no-daemon

# Android host unit tests (RobotBusAndroid.init without jniLibs)
test-android:
	cd bindings/android && ./gradlew test --no-daemon

# Back-compat aliases
kotlin-dev: java-dev
kotlin-android-native: android-native
kotlin-android: android-dev

# Java JVM smoke tests
test-java:
	cd bindings/java && mvn test

alias test-kotlin := test-java

# Run C++ binding tests (ephemeral in-process brokers)
test-cpp:
	cmake --build bindings/cpp/build --target cpp_tests
	./bindings/cpp/build/msgs_roundtrip
	./bindings/cpp/build/timer_spin
	./bindings/cpp/build/pub_sub_imu
	./bindings/cpp/build/service_set_bool
	./bindings/cpp/build/action_fibonacci
	./bindings/cpp/build/grpc_node
	./bindings/cpp/build/inproc_context

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

# Native Python integration (needs `just python-dev` first; skips if extension missing)
test-python-native:
	python3 bindings/python/tests/test_grpc_node.py
	python3 bindings/python/tests/test_inproc_context.py

# TypeScript smoke tests (msgs + GrpcNode guards; inproc skips without native addon)
test-typescript: gen-typescript
	cd bindings/typescript && npm test

# Local checks aligned with CI (codegen then rust + python/ts smoke)
ci: gen-all
	just test-rust
	just test-rust-minimal
	just test-python
	just test-typescript

# Performance harness (release); writes docs/perf-report.md
perf: gen-rust
	cargo run --release --bin robot_bus_perf --features grpc
