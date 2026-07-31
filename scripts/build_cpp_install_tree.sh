#!/usr/bin/env bash
# Build install tree for robot-bus C++ packages.
# Usage: scripts/build_cpp_install_tree.sh <dest> [version]
set -euo pipefail

DEST="${1:?dest}"
VERSION="${2:-0.1.0}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CPP="$ROOT/bindings/cpp"

# Protobuf stubs are gitignored; ensure they exist before cargo / cmake.
python3 "$ROOT/scripts/generate_cpp_msgs.py"
python3 "$ROOT/scripts/generate_rust_msgs.py"

rm -rf "$DEST"
mkdir -p "$DEST/usr/bin" "$DEST/usr/lib" "$DEST/usr/include" \
  "$DEST/usr/lib/pkgconfig" "$DEST/usr/lib/cmake/robot_bus"

# Broker
cargo build --release --manifest-path "$ROOT/Cargo.toml" --bin robot_bus_broker
cp -f "$ROOT/target/release/robot_bus_broker" "$DEST/usr/bin/"

# FFI (rename robot_bus_c → robot_bus)
cargo build --release --manifest-path "$CPP/native/Cargo.toml"
if [[ "$(uname -s)" == "Darwin" ]]; then
  cp -f "$CPP/native/target/release/librobot_bus_c.dylib" "$DEST/usr/lib/librobot_bus.dylib"
elif [[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* ]]; then
  cp -f "$CPP/native/target/release/robot_bus_c.dll" "$DEST/usr/bin/robot_bus.dll" || true
  cp -f "$CPP/native/target/release/robot_bus_c.dll.lib" "$DEST/usr/lib/robot_bus.lib" 2>/dev/null || true
else
  cp -f "$CPP/native/target/release/librobot_bus_c.so" "$DEST/usr/lib/librobot_bus.so"
fi

# Headers
cp -a "$CPP/include/." "$DEST/usr/include/"
cp -a "$CPP/generated/robot_bus" "$DEST/usr/include/"

# Msgs library via CMake (no tests)
BUILD_DIR="$CPP/build-package"
cmake -S "$CPP" -B "$BUILD_DIR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DROBOT_BUS_BUILD_TESTS=OFF \
  -DCMAKE_INSTALL_PREFIX="$DEST/usr" \
  ${CMAKE_PREFIX_PATH:+-DCMAKE_PREFIX_PATH="$CMAKE_PREFIX_PATH"}
JOBS="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 2)"
# GitHub Actions runners OOM when compiling ~130 protobuf .pb.cc units in parallel.
if [ "${CI:-}" = "true" ] && [ "$JOBS" -gt 2 ]; then JOBS=2; fi
cmake --build "$BUILD_DIR" --target robot_bus_msgs -j"$JOBS"
# Copy msgs lib from build dir
find "$BUILD_DIR" -name 'librobot_bus_msgs*' -o -name 'robot_bus_msgs.*' | while read -r f; do
  case "$f" in
    *.so*|*.dylib|*.dll|*.lib|*.a) cp -f "$f" "$DEST/usr/lib/" ;;
  esac
done

# pkg-config
sed -e "s|@CMAKE_INSTALL_PREFIX@|/usr|g" \
    -e "s|@CMAKE_INSTALL_LIBDIR@|lib|g" \
    -e "s|@CMAKE_INSTALL_INCLUDEDIR@|include|g" \
    -e "s|@PROJECT_VERSION@|${VERSION}|g" \
    "$CPP/cmake/robot_bus.pc.in" >"$DEST/usr/lib/pkgconfig/robot_bus.pc"

# Minimal CMake package config
cp "$CPP/cmake/robot_busConfig.cmake.in" "$DEST/usr/lib/cmake/robot_bus/robot_busConfig.cmake"
# Strip @PACKAGE_INIT@ for a usable installed file
sed -i.bak 's/@PACKAGE_INIT@//' "$DEST/usr/lib/cmake/robot_bus/robot_busConfig.cmake"
rm -f "$DEST/usr/lib/cmake/robot_bus/robot_busConfig.cmake.bak"
# Fix PACKAGE_PREFIX_DIR references roughly
sed -i.bak 's|\${PACKAGE_PREFIX_DIR}|/usr|g' "$DEST/usr/lib/cmake/robot_bus/robot_busConfig.cmake"
rm -f "$DEST/usr/lib/cmake/robot_bus/robot_busConfig.cmake.bak"

echo "install tree ready at $DEST"
