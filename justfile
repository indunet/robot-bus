# Lightweight task runner — does not replace cargo / maturin / pnpm.
# Install: https://github.com/casey/just
#
# Layout: Rust core at repo root; language SDKs under bindings/;
# benches/; tests/ for interop; console/ is broker monitoring UI;
# Console TANK panel pairs with in-process src/tank (session-acquired).

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

# Generate protobuf stubs for Android SDK under bindings/android/generated/
gen-android:
	ROBOT_BUS_JAVA_OUT=bindings/android/generated python3 scripts/generate_java_msgs.py

# Generate Rust prost/tonic stubs under src/generated/<pkg>/…/v1/<stem>.rs (gitignored)
gen-rust:
	python3 scripts/generate_rust_msgs.py

# Typed ROS↔protobuf topic mappers + ros-env-shim stubs
gen-topic-mappers:
	python3 scripts/generate_topic_mappers.py

# All language stubs (protoc 35.1)
gen-all: gen-rust proto gen-typescript gen-cpp gen-java gen-android

# Build and install the Python binding into the active venv (includes Web console)
python-dev: proto gen-rust
	if [[ ! -f assets/console/index.html ]]; then just console; fi
	cd bindings/python && maturin develop --features extension-module,ws,console --no-default-features

# Same as python-dev; Ros2Bridge is pure Python (rclpy). Source Humble/Jazzy first.
python-dev-ros2: proto gen-rust
	if [[ ! -f assets/console/index.html ]]; then just console; fi
	cd bindings/python && maturin develop --features extension-module,ws,console --no-default-features

# Build TypeScript native addon + JS bundle (includes Web console)
ts-dev: gen-typescript gen-rust
	if [[ ! -f assets/console/index.html ]]; then just console; fi
	cd bindings/typescript && npm install && npm run build:native && npm run build:ts

# Build C++ FFI + msgs library + tests
cpp-dev: gen-cpp gen-rust
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cmake -S bindings/cpp -B bindings/cpp/build -DCMAKE_BUILD_TYPE=Release
	cmake --build bindings/cpp/build -j

# Same as cpp-dev but enable native C++ ROS 2 bridge (requires sourced Humble or Jazzy).
cpp-dev-ros2: gen-cpp gen-rust
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cmake -S bindings/cpp -B bindings/cpp/build -DCMAKE_BUILD_TYPE=Release -DROBOT_BUS_ROS2=ON
	cmake --build bindings/cpp/build -j

# Build Java JVM binding with Maven (needs librobot_bus_c from cpp native)
java-dev: gen-java
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cd bindings/java && mvn test

# Install Java SDK into ~/.m2 (JVM consumers / local experiments)
java-install: gen-java
	cd bindings/java && mvn -DskipTests install

# Cross-compile librobot_bus_c for Android ABIs into bindings/android jniLibs
android-native:
	./scripts/build_android_native.sh

# Assemble Android AAR (self-contained Kotlin SDK; no java-install)
android-dev: gen-android android-native
	cd bindings/android && ./gradlew assembleRelease --no-daemon

# Android host unit tests (needs desktop librobot_bus_c for API tests)
test-android: gen-android
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cd bindings/android && ROBOT_BUS_NATIVE_DIR="$$(pwd)/../cpp/native/target/release" ./gradlew test --no-daemon

# Back-compat aliases
kotlin-dev: java-dev
kotlin-android-native: android-native
kotlin-android: android-dev

# Java JVM smoke tests
test-java:
	cd bindings/java && mvn test

# Run C++ binding tests (ephemeral in-process brokers)
test-cpp:
	cmake --build bindings/cpp/build --target cpp_tests
	./bindings/cpp/build/msgs_roundtrip
	./bindings/cpp/build/timer_spin
	./bindings/cpp/build/pub_sub_imu
	./bindings/cpp/build/service_set_bool
	./bindings/cpp/build/action_fibonacci
	./bindings/cpp/build/ws_node
	./bindings/cpp/build/inproc_context
	./bindings/cpp/build/federation_opts
	./bindings/cpp/build/node_parameters
	./bindings/cpp/build/ros2_bridge_stub

# --- examples/ (see examples/README.md) ---

examples-rust: gen-rust
	cargo build --examples

examples-python:
	#!/usr/bin/env bash
	set -euo pipefail
	if [[ -x .venv/bin/python ]]; then PY=.venv/bin/python; else PY=python3; fi
	"$PY" -m py_compile \
		examples/topic_imu/python/listener.py \
		examples/topic_imu/python/talker.py \
		examples/service_set_bool/python/server.py \
		examples/service_set_bool/python/client.py \
		examples/action_fibonacci/python/server.py \
		examples/action_fibonacci/python/client.py \
		examples/ros2_bridge/python/builtin.py \
		examples/ros2_bridge/python/custom_add_two_ints.py

examples-cpp: gen-cpp gen-rust
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cmake -S bindings/cpp -B bindings/cpp/build -DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_PREFIX_PATH="${CMAKE_PREFIX_PATH:-/usr/local}" \
		-DCMAKE_FIND_PACKAGE_PREFER_CONFIG=ON \
		-DROBOT_BUS_BUILD_EXAMPLES=ON
	cmake --build bindings/cpp/build --target cpp_examples -j2

# C++ examples including Ros2Bridge (source Humble/Jazzy first).
examples-cpp-ros2: gen-cpp gen-rust
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	cmake -S bindings/cpp -B bindings/cpp/build -DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_PREFIX_PATH="${CMAKE_PREFIX_PATH:-/usr/local}" \
		-DCMAKE_FIND_PACKAGE_PREFER_CONFIG=ON \
		-DROBOT_BUS_BUILD_EXAMPLES=ON \
		-DROBOT_BUS_ROS2=ON
	cmake --build bindings/cpp/build --target cpp_examples -j2

# End-to-end topic smoke (Python talker/listener against an ephemeral broker).
examples-topic:
	#!/usr/bin/env bash
	set -euo pipefail
	if [[ -x .venv/bin/python ]]; then PY=.venv/bin/python; else PY=python3; fi
	"$PY" scripts/run_examples_topic_smoke.py

examples: examples-rust examples-python examples-cpp examples-topic

# Build console UI into assets/console/ for rust-embed (gitignored; run before cargo with `console`)
console:
	./scripts/build_console.sh

# TANK is in-process now: open the console TANK window (POST /api/v1/tank/session).
tank:
	@echo "tank runs inside robot_bus_broker when a console TANK session is open."
	@echo "Start the broker, then open the sidebar TANK window (or /tank)."

# Rust tests (default features)
test-rust: gen-rust
	cargo test

# Rust tests without default features
test-rust-minimal: gen-rust
	cargo test --no-default-features

# Pure-Python message / typed-API smoke tests (no native extension required)
test-python: proto
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_msgs_roundtrip.py
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_typed_api.py
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_ros2_bridge_builder.py
	PYTHONPATH=bindings/python python3 bindings/python/tests/test_ros2_bridge_image_mapper.py

# Native Python integration (requires `just python-dev`; fails if extension missing)
test-python-native:
	python3 bindings/python/tests/test_ws_node.py
	python3 bindings/python/tests/test_inproc_context.py
	python3 bindings/python/tests/test_federation_opts.py
	python3 bindings/python/tests/test_console.py

# Cross-language interop matrix (6 language-pair scenarios). Missing peers fail.
# Needs `just python-dev`, console assets, protobuf C++ runtime, Java, Node/napi.
test-interop: gen-rust console
	#!/usr/bin/env bash
	set -euo pipefail
	cargo build --quiet --bin robot_bus_interop
	cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
	python3 scripts/generate_cpp_msgs.py
	cmake -S bindings/cpp -B bindings/cpp/build -DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_PREFIX_PATH="${CMAKE_PREFIX_PATH:-/usr/local}" \
		-DCMAKE_FIND_PACKAGE_PREFER_CONFIG=ON \
		-DROBOT_BUS_BUILD_TESTS=ON
	cmake --build bindings/cpp/build --target interop_peer -j2
	if [[ -f bindings/cpp/native/target/release/librobot_bus_c.so ]]; then
		ln -sfn "$(pwd)/bindings/cpp/native/target/release/librobot_bus_c.so" \
			bindings/cpp/build/librobot_bus.so
	fi
	python3 scripts/generate_java_msgs.py
	(cd bindings/java && mvn -q test-compile dependency:build-classpath -Dmdep.outputFile=target/interop-classpath.txt)
	python3 scripts/generate_typescript_msgs.py
	(cd bindings/typescript && npm ci && npm run build:native && npm run build:ts)
	if [[ -x .venv/bin/python ]]; then PY=.venv/bin/python; else PY=python3; fi
	"$PY" tests/interop/run.py

# TypeScript smoke tests (msgs + WsNode guards; inproc skips without native addon)
test-typescript: gen-typescript
	cd bindings/typescript && npm test

# Local checks aligned with CI smoke (codegen then rust + python/ts).
# Native Python / Java / interop gates: just python-dev && just test-python-native;
# just test-java; just test-interop
ci: gen-all
	just test-rust
	just test-rust-minimal
	just test-python
	just test-typescript

# Performance harness (release); writes docs/zh/perf-report.md and docs/en/perf-report.md
perf: gen-rust
	cargo run --release --bin robot_bus_perf --features ws

# ROS 2 comparison benches (Docker container `ros2`); writes docs/{zh,en}/ros2-perf-report.md
perf-ros2:
	./benches/ros2_perf/run.sh

# ROS ↔ bus bridge benches (Docker container `ros2`); writes docs/{zh,en}/ros2-bridge-perf-report.md
perf-ros2-bridge:
	./benches/ros2_bridge_perf/run.sh

# Typecheck ros2 bridge without a full ROS install (rclrs use_ros_shim)
check-ros2-shim:
	RUSTFLAGS='--cfg ros_distro="humble"' cargo check --features ros2-shim
	RUSTFLAGS='--cfg ros_distro="humble"' cargo check --manifest-path bindings/cpp/native/Cargo.toml --features ros2-shim
	RUSTFLAGS='--cfg ros_distro="humble"' cargo check --lib --no-default-features --features extension-module,ws,ros2-shim
