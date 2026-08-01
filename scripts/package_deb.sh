#!/usr/bin/env bash
# Assemble a Debian package for robot-bus C++ SDK + broker.
# Usage: scripts/package_deb.sh <version> <arch> <staging-prefix> <out-deb> [variant]
#   arch: amd64 | arm64
#   staging-prefix: directory that already has usr/bin, usr/lib, usr/include layout
#   variant: main (default) | ros2-humble | ros2-jazzy
set -euo pipefail

VERSION="${1:?version}"
ARCH="${2:?arch}"
STAGING="${3:?staging}"
OUT_DEB="${4:?out-deb}"
VARIANT="${5:-main}"

PKG_ROOT="$(mktemp -d)"
trap 'rm -rf "$PKG_ROOT"' EXIT

mkdir -p "$PKG_ROOT/DEBIAN"
cp -a "$STAGING/." "$PKG_ROOT/"

# Legacy package names (robot-bus-cpp*) are listed in Conflicts/Replaces so
# upgrading from older releases replaces the previous Debian package cleanly.
case "$VARIANT" in
  main)
    PACKAGE_NAME="robot-bus"
    DEPENDS="libzmq5"
    CONFLICTS="Conflicts: robot-bus-cpp, robot-bus-ros2-humble, robot-bus-ros2-jazzy, robot-bus-cpp-ros2-humble, robot-bus-cpp-ros2-jazzy
Replaces: robot-bus-cpp"
    DESCRIPTION_SHORT="robot-bus C/C++ SDK and broker"
    DESCRIPTION_LONG=" ZeroMQ message bus with ROS-style APIs. Ships shared libraries
 (including bundled libprotobuf matching the generated msgs),
 headers, CMake/pkg-config files, and the robot_bus_broker binary.
 Does not include the ROS 2 bridge (see robot-bus-ros2-humble /
 robot-bus-ros2-jazzy)."
    ;;
  ros2-humble)
    PACKAGE_NAME="robot-bus-ros2-humble"
    DEPENDS="libzmq5, ros-humble-rcl, ros-humble-std-msgs, ros-humble-sensor-msgs, ros-humble-std-srvs"
    CONFLICTS="Conflicts: robot-bus, robot-bus-cpp, robot-bus-ros2-jazzy, robot-bus-cpp-ros2-humble, robot-bus-cpp-ros2-jazzy
Provides: robot-bus
Replaces: robot-bus, robot-bus-cpp, robot-bus-cpp-ros2-humble"
    DESCRIPTION_SHORT="robot-bus C/C++ SDK with ROS 2 Humble bridge"
    DESCRIPTION_LONG=" Same as robot-bus, plus in-process ROS 2 topic/service bridge
 (Ros2Bridge) linked against system ROS 2 Humble. Requires a sourced
 Humble environment at runtime. Does not vendor rcl/RMW/DDS.
 Linux only — there is no Windows/macOS ros2 package."
    ;;
  ros2-jazzy)
    PACKAGE_NAME="robot-bus-ros2-jazzy"
    DEPENDS="libzmq5, ros-jazzy-rcl, ros-jazzy-std-msgs, ros-jazzy-sensor-msgs, ros-jazzy-std-srvs"
    CONFLICTS="Conflicts: robot-bus, robot-bus-cpp, robot-bus-ros2-humble, robot-bus-cpp-ros2-humble, robot-bus-cpp-ros2-jazzy
Provides: robot-bus
Replaces: robot-bus, robot-bus-cpp, robot-bus-cpp-ros2-jazzy"
    DESCRIPTION_SHORT="robot-bus C/C++ SDK with ROS 2 Jazzy bridge"
    DESCRIPTION_LONG=" Same as robot-bus, plus in-process ROS 2 topic/service bridge
 (Ros2Bridge) linked against system ROS 2 Jazzy. Requires a sourced
 Jazzy environment at runtime. Does not vendor rcl/RMW/DDS.
 Linux only — there is no Windows/macOS ros2 package."
    ;;
  *)
    echo "error: unknown variant '$VARIANT' (main|ros2-humble|ros2-jazzy)" >&2
    exit 1
    ;;
esac

{
  echo "Package: ${PACKAGE_NAME}"
  echo "Version: ${VERSION}"
  echo "Section: libs"
  echo "Priority: optional"
  echo "Architecture: ${ARCH}"
  echo "Maintainer: deng_ran <deng_ran@aliyun.com>"
  echo "Depends: ${DEPENDS}"
  if [[ -n "$CONFLICTS" ]]; then
    printf '%s\n' "$CONFLICTS"
  fi
  echo "Description: ${DESCRIPTION_SHORT}"
  printf '%s\n' "$DESCRIPTION_LONG"
  echo "Homepage: https://github.com/indunet/robot-bus"
} >"$PKG_ROOT/DEBIAN/control"

# Ensure executable bit on broker if present
if [[ -f "$PKG_ROOT/usr/bin/robot_bus_broker" ]]; then
  chmod 755 "$PKG_ROOT/usr/bin/robot_bus_broker"
fi

dpkg-deb --build --root-owner-group "$PKG_ROOT" "$OUT_DEB"
echo "wrote $OUT_DEB ($VARIANT)"
