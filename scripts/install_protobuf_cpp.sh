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
  # Multi-config VS generator (MSVC is the default on windows-latest runners).
  CMAKE_ARGS+=(-G "Visual Studio 17 2022" -A x64)
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
