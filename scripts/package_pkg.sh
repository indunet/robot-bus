#!/usr/bin/env bash
# Assemble a macOS .pkg for robot-bus C++ SDK + broker (Apple Silicon).
# Usage: scripts/package_pkg.sh <version> <staging-prefix> <out-pkg>
#   staging-prefix: directory with usr/bin, usr/lib, usr/include (from
#                   build_cpp_install_tree.sh). Remapped to /usr/local in the pkg.
set -euo pipefail

VERSION="${1:?version}"
STAGING="${2:?staging}"
OUT_PKG="${3:?out-pkg}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "package_pkg.sh must run on macOS" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

PAYLOAD="$WORK/payload"
mkdir -p "$PAYLOAD/usr/local"
cp -a "$STAGING/usr/." "$PAYLOAD/usr/local/"

# Rewrite install paths from /usr → /usr/local for pkg-config / CMake helpers.
if [[ -f "$PAYLOAD/usr/local/lib/pkgconfig/robot_bus.pc" ]]; then
  sed -i.bak 's|^prefix=/usr$|prefix=/usr/local|; s|/usr/lib|/usr/local/lib|g; s|/usr/include|/usr/local/include|g' \
    "$PAYLOAD/usr/local/lib/pkgconfig/robot_bus.pc"
  rm -f "$PAYLOAD/usr/local/lib/pkgconfig/robot_bus.pc.bak"
fi
if [[ -f "$PAYLOAD/usr/local/lib/cmake/robot_bus/robot_busConfig.cmake" ]]; then
  sed -i.bak 's|/usr|/usr/local|g' \
    "$PAYLOAD/usr/local/lib/cmake/robot_bus/robot_busConfig.cmake"
  rm -f "$PAYLOAD/usr/local/lib/cmake/robot_bus/robot_busConfig.cmake.bak"
fi

if [[ -f "$PAYLOAD/usr/local/bin/robot_bus_broker" ]]; then
  chmod 755 "$PAYLOAD/usr/local/bin/robot_bus_broker"
fi

# Point bundled dylibs / binaries at /usr/local/lib (absolute ids).
fix_install_names() {
  local path="$1"
  [[ -f "$path" ]] || return 0
  local base dep
  case "$path" in
    *.dylib)
      base="$(basename "$path")"
      install_name_tool -id "/usr/local/lib/${base}" "$path" 2>/dev/null || true
      ;;
  esac
  while IFS= read -r dep; do
    case "$dep" in
      /usr/lib/*|/System/*) continue ;;
      *.dylib)
        base="$(basename "$dep")"
        if [[ -f "$PAYLOAD/usr/local/lib/$base" ]]; then
          install_name_tool -change "$dep" "/usr/local/lib/$base" "$path" 2>/dev/null || true
        fi
        ;;
    esac
  done < <(otool -L "$path" 2>/dev/null | awk 'NR>1 {print $1}')
}

shopt -s nullglob
for f in "$PAYLOAD/usr/local/lib/"*.dylib "$PAYLOAD/usr/local/bin/"*; do
  [[ -f "$f" ]] || continue
  file "$f" | grep -q 'Mach-O' || continue
  fix_install_names "$f"
done
shopt -u nullglob

pkgbuild \
  --root "$PAYLOAD" \
  --identifier "org.indunet.robot-bus" \
  --version "$VERSION" \
  --install-location "/" \
  "$OUT_PKG"

echo "wrote $OUT_PKG"
