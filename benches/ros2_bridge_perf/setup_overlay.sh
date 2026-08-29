#!/usr/bin/env bash
# Fallback: if the distro install already has rust IDL for std_msgs /
# sensor_msgs / example_interfaces, this is a no-op. Otherwise colcon-build
# those packages with rosidl_generator_rs (older images).
set -euo pipefail

WS="${ROS2_RUST_WS:-/tmp/ros2_rust_ws}"
DISTRO="${ROS_DISTRO:-humble}"

has_overlay() {
  local p
  IFS=':' read -ra paths <<< "${AMENT_PREFIX_PATH:-}"
  for p in "${paths[@]}"; do
    if [[ -f "${p}/share/sensor_msgs/rust/Cargo.toml" \
       && -f "${p}/share/std_msgs/rust/Cargo.toml" \
       && -f "${p}/share/example_interfaces/rust/Cargo.toml" ]]; then
      return 0
    fi
  done
  return 1
}

source_ros() {
  set +u
  # shellcheck disable=SC1091
  source "/opt/ros/${DISTRO}/setup.bash"
  if [[ -f "${WS}/install/setup.bash" ]]; then
    # shellcheck disable=SC1091
    source "${WS}/install/setup.bash"
  fi
  set -u
}

source_ros
if has_overlay; then
  echo "rust IDL overlay already on AMENT_PREFIX_PATH"
  exit 0
fi

echo "==> preparing rust IDL overlay at ${WS}"
mkdir -p "${WS}/src"
cd "${WS}"
clone_if_missing() {
  local dest="$1" url="$2" branch="${3:-}"
  if [[ -d "${dest}/.git" ]]; then
    return 0
  fi
  if [[ -n "${branch}" ]]; then
    git clone --depth 1 -b "${branch}" "${url}" "${dest}"
  else
    git clone --depth 1 "${url}" "${dest}"
  fi
}

clone_if_missing src/common_interfaces https://github.com/ros2/common_interfaces.git "${DISTRO}"
clone_if_missing src/example_interfaces https://github.com/ros2/example_interfaces.git "${DISTRO}"
clone_if_missing src/rcl_interfaces https://github.com/ros2/rcl_interfaces.git "${DISTRO}"
clone_if_missing src/rosidl_core https://github.com/ros2/rosidl_core.git "${DISTRO}"
clone_if_missing src/rosidl_defaults https://github.com/ros2/rosidl_defaults.git "${DISTRO}"
clone_if_missing src/unique_identifier_msgs https://github.com/ros2/unique_identifier_msgs.git "${DISTRO}"
clone_if_missing src/rosidl_rust https://github.com/ros2-rust/rosidl_rust.git

source_ros
colcon build --symlink-install
set +u
# shellcheck disable=SC1091
source "${WS}/install/setup.bash"
set -u
if ! has_overlay; then
  echo "overlay build finished but share/*/rust is still missing" >&2
  exit 1
fi
echo "overlay ready: ${WS}/install"
