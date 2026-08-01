#!/usr/bin/env bash
# Install protobuf C++ runtime matching EXPECTED_PROTOC_VERSION (35.1).
# Usage: scripts/install_protobuf_cpp.sh [prefix]
set -euo pipefail

PREFIX="${1:-/usr/local}"
VERSION="${PROTOBUF_VERSION:-35.1}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Git Bash / MSYS on Windows
is_windows() {
  case "$(uname -s 2>/dev/null || true)" in
    MINGW*|MSYS*|CYGWIN*) return 0 ;;
  esac
  [[ "${OS:-}" == "Windows_NT" ]]
}

# Pick a multi-config VS generator for the installed toolchain.
# windows-latest (mid-2026+) ships VS 2026 only; windows-2022 still has VS 2022.
windows_vs_generator() {
  if [[ -n "${CMAKE_GENERATOR:-}" ]]; then
    printf '%s\n' "$CMAKE_GENERATOR"
    return
  fi
  local vsroot="/c/Program Files/Microsoft Visual Studio"
  if [[ -d "$vsroot/18" ]]; then
    echo "Visual Studio 18 2026"
  elif [[ -d "$vsroot/2022" ]]; then
    echo "Visual Studio 17 2022"
  else
    echo "error: no Visual Studio install found under $vsroot (tried 18 / 2022)" >&2
    return 1
  fi
}

curl -fsSL -o "$WORK/protobuf.tar.gz" \
  "https://github.com/protocolbuffers/protobuf/releases/download/v${VERSION}/protobuf-${VERSION}.tar.gz"
tar -xzf "$WORK/protobuf.tar.gz" -C "$WORK"
SRC="$WORK/protobuf-${VERSION}"

JOBS="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"

CMAKE_ARGS=(
  -S "$SRC"
  -B "$WORK/build"
  -DCMAKE_INSTALL_PREFIX="$PREFIX"
  -DCMAKE_CXX_STANDARD=17
  -Dprotobuf_BUILD_TESTS=OFF
  -Dprotobuf_BUILD_SHARED_LIBS=ON
  -Dprotobuf_INSTALL=ON
)

if is_windows; then
  VS_GEN="$(windows_vs_generator)"
  echo "Using CMake generator: ${VS_GEN}"
  CMAKE_ARGS+=(-G "$VS_GEN" -A x64)
  cmake "${CMAKE_ARGS[@]}"
  cmake --build "$WORK/build" --config Release -j"$JOBS"
  cmake --install "$WORK/build" --config Release
else
  CMAKE_ARGS+=(-DCMAKE_BUILD_TYPE=Release)
  cmake "${CMAKE_ARGS[@]}"
  cmake --build "$WORK/build" -j"$JOBS"
  cmake --install "$WORK/build"
fi

# Prefer this protoc if present
if [[ -x "$PREFIX/bin/protoc" ]]; then
  "$PREFIX/bin/protoc" --version
elif [[ -x "$PREFIX/bin/protoc.exe" ]]; then
  "$PREFIX/bin/protoc.exe" --version
fi

echo "protobuf ${VERSION} installed to ${PREFIX}"
