#!/usr/bin/env bash
# Build librobot_bus_c.so for Android ABIs into
# bindings/android/src/main/jniLibs/<abi>/
#
# Requires:
#   - ANDROID_HOME or ANDROID_SDK_ROOT
#   - NDK (r26 recommended): sdkmanager "ndk;26.3.11579264"
#   - rustup targets + cargo-ndk
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NATIVE_CRATE="$ROOT/bindings/cpp/native"
JNI_OUT="$ROOT/bindings/android/src/main/jniLibs"
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

build_abi() {
  local abi="$1"
  local clang_triple="$2"

  echo "=== Building robot_bus_c for $abi ==="
  (
    cd "$NATIVE_CRATE"
    # Ensure Rust msgs exist for dependent crate features
    if [[ ! -f "$ROOT/crates/robot-bus-msgs/src/generated/sensor_msgs/msg/v1/imu.rs" ]]; then
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

  local dynamic
  dynamic="$("$TOOLCHAIN/bin/llvm-readelf" -d "$JNI_OUT/$abi/librobot_bus_c.so")"
  if [[ "$dynamic" == *libzmq.so* ]]; then
    echo "Expected statically linked libzmq in robot_bus_c for $abi" >&2
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
build_abi "arm64-v8a" "aarch64-linux-android"
build_abi "armeabi-v7a" "arm-linux-androideabi"
build_abi "x86_64" "x86_64-linux-android"

echo "Android jniLibs ready under $JNI_OUT"
find "$JNI_OUT" -type f | sort
