#!/usr/bin/env bash
# Write a relocatable robot_bus CMake package (config + version file).
# Usage: scripts/stage_robot_bus_cmake_config.sh <dest-dir> <version>
#
# configure_package_config_file emits a prefix computed from this file's
# location (<prefix>/lib/cmake/robot_bus/), so the same files work under /usr,
# /usr/local, C:/Program Files/robot-bus, or a FetchContent _deps directory.
set -euo pipefail

DEST="${1:?dest cmake dir}"
VERSION="${2:?version}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$DEST"
# The generated CMake source is read by native Windows CMake under Git Bash.
if command -v cygpath >/dev/null 2>&1; then
  ROOT="$(cygpath -am "$ROOT")"
  DEST="$(cygpath -am "$DEST")"
fi
cat >"$WORK/CMakeLists.txt" <<EOF
cmake_minimum_required(VERSION 3.16)
project(robot_bus_cmake_config LANGUAGES NONE)
include(CMakePackageConfigHelpers)
configure_package_config_file(
  "${ROOT}/bindings/cpp/cmake/robot_busConfig.cmake.in"
  "${DEST}/robot_busConfig.cmake"
  INSTALL_DESTINATION lib/cmake/robot_bus)
EOF
cmake -S "$WORK" -B "$WORK/build" >/dev/null
cmake -DVERSION="$VERSION" -DOUT="$DEST/robot_busConfigVersion.cmake" \
  -P "$ROOT/bindings/cpp/cmake/write_config_version.cmake"
echo "wrote relocatable CMake package in $DEST"
