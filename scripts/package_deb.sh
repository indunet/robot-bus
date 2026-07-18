#!/usr/bin/env bash
# Assemble a Debian package for robot-bus C++ SDK + broker.
# Usage: scripts/package_deb.sh <version> <arch> <staging-prefix> <out-deb>
#   arch: amd64 | arm64
#   staging-prefix: directory that already has usr/bin, usr/lib, usr/include layout
set -euo pipefail

VERSION="${1:?version}"
ARCH="${2:?arch}"
STAGING="${3:?staging}"
OUT_DEB="${4:?out-deb}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PKG_ROOT="$(mktemp -d)"
trap 'rm -rf "$PKG_ROOT"' EXIT

mkdir -p "$PKG_ROOT/DEBIAN"
cp -a "$STAGING/." "$PKG_ROOT/"

cat >"$PKG_ROOT/DEBIAN/control" <<EOF
Package: robot-bus
Version: ${VERSION}
Section: libs
Priority: optional
Architecture: ${ARCH}
Maintainer: deng_ran <deng_ran@aliyun.com>
Depends: libzmq5
Description: robot-bus C/C++ SDK and broker
 ZeroMQ message bus with ROS-style APIs. Ships shared libraries
 (including bundled libprotobuf matching the generated msgs),
 headers, CMake/pkg-config files, and the robot_bus_broker binary.
Homepage: https://github.com/indunet/robot-bus
EOF

# Ensure executable bit on broker if present
if [[ -f "$PKG_ROOT/usr/bin/robot_bus_broker" ]]; then
  chmod 755 "$PKG_ROOT/usr/bin/robot_bus_broker"
fi

dpkg-deb --build --root-owner-group "$PKG_ROOT" "$OUT_DEB"
echo "wrote $OUT_DEB"
