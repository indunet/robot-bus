#!/usr/bin/env bash
# Build librobot_bus_c.so for Android ABIs into
# bindings/android/src/main/jniLibs/<abi>/
#
# Requires:
#   - ANDROID_HOME or ANDROID_SDK_ROOT
#   - NDK (r26 recommended): sdkmanager "ndk;26.3.11579264"
#   - rustup targets + cargo-ndk
#   - cmake, curl, unzip
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NATIVE_CRATE="$ROOT/bindings/cpp/native"
JNI_OUT="$ROOT/bindings/android/src/main/jniLibs"
DEPS="$ROOT/bindings/android/.android-native-deps"
LIBZMQ_VER="${LIBZMQ_VER:-4.3.5}"
API_LEVEL="${ANDROID_API_LEVEL:-24}"

ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [[ -z "$ANDROID_HOME" && -d "${HOME}/Library/Android/sdk" ]]; then
  ANDROID_HOME="${HOME}/Library/Android/sdk"
fi
if [[ -z "$ANDROID_HOME" || ! -d "$ANDROID_HOME" ]]; then
  echo "ANDROID_HOME / ANDROID_SDK_ROOT not set" >&2
  exit 1
fi
export ANDROID_HOME

# Prefer NDK 26.x if present
if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
  if [[ -d "$ANDROID_HOME/ndk" ]]; then
    ANDROID_NDK_HOME="$(ls -d "$ANDROID_HOME/ndk"/26.* 2>/dev/null | sort -V | tail -n1 || true)"
    if [[ -z "$ANDROID_NDK_HOME" ]]; then
      ANDROID_NDK_HOME="$(ls -d "$ANDROID_HOME/ndk"/* 2>/dev/null | sort -V | tail -n1 || true)"
    fi
  fi
fi
if [[ -z "${ANDROID_NDK_HOME:-}" || ! -d "$ANDROID_NDK_HOME" ]]; then
  echo "Android NDK not found. Install e.g.: sdkmanager \"ndk;26.3.11579264\"" >&2
  exit 1
fi
export ANDROID_NDK_HOME
echo "Using NDK: $ANDROID_NDK_HOME"

HOST_TAG="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
case "$HOST_TAG" in
  darwin-arm64) HOST_TAG="darwin-x86_64" ;; # NDK still uses darwin-x86_64 layout on Apple Silicon
  linux-aarch64) HOST_TAG="linux-x86_64" ;;
esac
# On Apple Silicon the prebuilt folder is darwin-x86_64 (Rosetta) or darwin-arm64 on newer NDKs
if [[ ! -d "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG" ]]; then
  if [[ -d "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-arm64" ]]; then
    HOST_TAG="darwin-arm64"
  elif [[ -d "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64" ]]; then
    HOST_TAG="darwin-x86_64"
  elif [[ -d "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64" ]]; then
    HOST_TAG="linux-x86_64"
  else
    echo "Cannot find NDK prebuilt toolchain under $ANDROID_NDK_HOME/toolchains/llvm/prebuilt" >&2
    ls "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt" >&2 || true
    exit 1
  fi
fi
TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG"
echo "Using toolchain: $TOOLCHAIN"

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "Installing cargo-ndk..."
  cargo install cargo-ndk --locked
fi

rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

mkdir -p "$DEPS"
LIBZMQ_SRC="$DEPS/libzmq-$LIBZMQ_VER"
if [[ ! -d "$LIBZMQ_SRC" ]]; then
  echo "Fetching libzmq $LIBZMQ_VER..."
  curl -fsSL "https://github.com/zeromq/libzmq/releases/download/v${LIBZMQ_VER}/zeromq-${LIBZMQ_VER}.tar.gz" \
    | tar -xz -C "$DEPS"
  mv "$DEPS/zeromq-$LIBZMQ_VER" "$LIBZMQ_SRC"
fi

# ABI -> rust triple, clang prefix, android abi dir
build_abi() {
  local abi="$1"
  local triple="$2"
  local clang_triple="$3"
  local zmq_prefix="$DEPS/libzmq-install/$abi"
  local build_dir="$DEPS/libzmq-build/$abi"

  mkdir -p "$build_dir" "$zmq_prefix"
  if [[ ! -f "$zmq_prefix/lib/libzmq.a" && ! -f "$zmq_prefix/lib/libzmq.so" ]]; then
    echo "=== Building libzmq for $abi ==="
    cmake -S "$LIBZMQ_SRC" -B "$build_dir" \
      -DCMAKE_TOOLCHAIN_FILE="$ANDROID_NDK_HOME/build/cmake/android.toolchain.cmake" \
      -DANDROID_ABI="$abi" \
      -DANDROID_PLATFORM="android-$API_LEVEL" \
      -DANDROID_STL=c++_shared \
      -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX="$zmq_prefix" \
      -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
      -DBUILD_SHARED=OFF \
      -DBUILD_STATIC=ON \
      -DBUILD_TESTS=OFF \
      -DWITH_PERF_TOOL=OFF \
      -DWITH_DOCS=OFF \
      -DENABLE_CPACK=OFF \
      -DENABLE_DRAFTS=OFF
    cmake --build "$build_dir" -j"$(sysctl -n hw.ncpu 2>/dev/null || nproc)"
    cmake --install "$build_dir"
  else
    echo "=== Reusing libzmq for $abi at $zmq_prefix ==="
  fi

  mkdir -p "$zmq_prefix/lib/pkgconfig"
  cat >"$zmq_prefix/lib/pkgconfig/libzmq.pc" <<EOF
prefix=$zmq_prefix
exec_prefix=\${prefix}
libdir=\${prefix}/lib
includedir=\${prefix}/include

Name: libzmq
Description: ZeroMQ library (Android static)
Version: $LIBZMQ_VER
Libs: -L\${libdir} -lzmq
Libs.private: -lc++_shared -lm
Cflags: -I\${includedir}
EOF

  echo "=== Building robot_bus_c for $abi ($triple) ==="
  export PKG_CONFIG_ALLOW_CROSS=1
  export PKG_CONFIG_PATH="$zmq_prefix/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
  export ZMQ_LIB_DIR="$zmq_prefix/lib"
  export ZMQ_INCLUDE_DIR="$zmq_prefix/include"
  export LIBZMQ_PREFIX="$zmq_prefix"

  (
    cd "$NATIVE_CRATE"
    # Ensure Rust msgs exist for dependent crate features
    if [[ ! -d "$ROOT/src/msgs/generated" ]]; then
      python3 "$ROOT/scripts/generate_rust_msgs.py"
    fi
    cargo ndk -t "$abi" -P "$API_LEVEL" --link-libcxx-shared -o "$JNI_OUT" build --release
  )

  # cargo-ndk names the output from [lib] name = robot_bus_c → librobot_bus_c.so
  if [[ ! -f "$JNI_OUT/$abi/librobot_bus_c.so" ]]; then
    echo "Expected $JNI_OUT/$abi/librobot_bus_c.so missing" >&2
    find "$JNI_OUT" -type f 2>/dev/null || true
    exit 1
  fi

  # Bundle c++_shared if linked dynamically (common with NDK STL)
  local cpp_shared="$TOOLCHAIN/sysroot/usr/lib/$clang_triple/libc++_shared.so"
  if [[ ! -f "$cpp_shared" ]]; then
    cpp_shared="$(find "$TOOLCHAIN" -name 'libc++_shared.so' | grep "$clang_triple" | head -n1 || true)"
  fi
  if [[ -n "$cpp_shared" && -f "$cpp_shared" ]]; then
    cp -f "$cpp_shared" "$JNI_OUT/$abi/"
  fi
}

mkdir -p "$JNI_OUT"
build_abi "arm64-v8a" "aarch64-linux-android" "aarch64-linux-android"
build_abi "armeabi-v7a" "armv7-linux-androideabi" "arm-linux-androideabi"
build_abi "x86_64" "x86_64-linux-android" "x86_64-linux-android"

echo "Android jniLibs ready under $JNI_OUT"
find "$JNI_OUT" -type f | sort
