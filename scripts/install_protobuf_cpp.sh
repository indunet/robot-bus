#!/usr/bin/env bash
# Install protobuf C++ runtime matching EXPECTED_PROTOC_VERSION (35.1).
# Usage: scripts/install_protobuf_cpp.sh [prefix]
set -euo pipefail

PREFIX="${1:-/usr/local}"
VERSION="${PROTOBUF_VERSION:-35.1}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

curl -fsSL -o "$WORK/protobuf.tar.gz" \
  "https://github.com/protocolbuffers/protobuf/releases/download/v${VERSION}/protobuf-${VERSION}.tar.gz"
tar -xzf "$WORK/protobuf.tar.gz" -C "$WORK"
SRC="$WORK/protobuf-${VERSION}"

cmake -S "$SRC" -B "$WORK/build" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$PREFIX" \
  -DCMAKE_CXX_STANDARD=17 \
  -Dprotobuf_BUILD_TESTS=OFF \
  -Dprotobuf_BUILD_SHARED_LIBS=ON \
  -Dprotobuf_INSTALL=ON
cmake --build "$WORK/build" -j"$(nproc 2>/dev/null || sysctl -n hw.ncpu || echo 4)"
cmake --install "$WORK/build"

# Prefer this protoc if present
if [[ -x "$PREFIX/bin/protoc" ]]; then
  "$PREFIX/bin/protoc" --version
fi

echo "protobuf ${VERSION} installed to ${PREFIX}"
