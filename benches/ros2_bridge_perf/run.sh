#!/usr/bin/env bash
# Build and run ROS↔bus bridge perf inside Docker container `ros2`.
# Writes docs/zh/ros2-bridge-perf-report.md and docs/en/ros2-bridge-perf-report.md.
set -euo pipefail

PERF_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ "$(basename "$(dirname "${PERF_DIR}")")" == "benches" ]]; then
  ROOT="$(cd "${PERF_DIR}/../.." && pwd)"
else
  ROOT="$(cd "${PERF_DIR}/.." && pwd)"
fi
CONTAINER="${ROS2_PERF_CONTAINER:-ros2}"
WS_IN_CONTAINER="${ROS2_BRIDGE_PERF_WS:-/tmp/robot-bus}"

usage() {
  cat <<'EOF'
Usage: benches/ros2_bridge_perf/run.sh [--local]

  (default)  Sync this repo into Docker container `ros2`, overlay + cargo bench, pull reports.
  --local    Already inside the container (or ROS+overlay sourced); run cargo bench here.
EOF
}

LOCAL=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --local) LOCAL=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

run_local() {
  local root="$1"
  set +u
  # shellcheck disable=SC1091
  source /opt/ros/humble/setup.bash
  set -u
  bash "${root}/benches/ros2_bridge_perf/setup_overlay.sh"
  set +u
  # shellcheck disable=SC1091
  source /opt/ros/humble/setup.bash
  if [[ -f "${ROS2_RUST_WS:-/tmp/ros2_rust_ws}/install/setup.bash" ]]; then
    # shellcheck disable=SC1091
    source "${ROS2_RUST_WS:-/tmp/ros2_rust_ws}/install/setup.bash"
  fi
  set -u
  cd "${root}"
  RUSTFLAGS="${RUSTFLAGS:---cfg ros_distro=\"humble\"}" \
    cargo run --release --bin ros2_bridge_perf --features ros2
}

if [[ "${LOCAL}" -eq 1 ]]; then
  run_local "${ROOT}"
  exit 0
fi

if ! docker inspect "${CONTAINER}" >/dev/null 2>&1; then
  echo "Docker container '${CONTAINER}' not found." >&2
  exit 1
fi
if [[ "$(docker inspect -f '{{.State.Running}}' "${CONTAINER}")" != "true" ]]; then
  echo "Docker container '${CONTAINER}' is not running." >&2
  exit 1
fi

echo "==> syncing ${ROOT} -> ${CONTAINER}:${WS_IN_CONTAINER}"
docker exec "${CONTAINER}" mkdir -p "${WS_IN_CONTAINER}"
docker cp "${ROOT}/." "${CONTAINER}:${WS_IN_CONTAINER}"

echo "==> overlay + bench inside ${CONTAINER}"
docker exec \
  -e ROS2_BRIDGE_PERF_IMAGE_WIDTH="${ROS2_BRIDGE_PERF_IMAGE_WIDTH:-}" \
  -e ROS2_BRIDGE_PERF_IMAGE_HEIGHT="${ROS2_BRIDGE_PERF_IMAGE_HEIGHT:-}" \
  -e ROS2_BRIDGE_PERF_MAX_LOSS_PCT="${ROS2_BRIDGE_PERF_MAX_LOSS_PCT:-}" \
  -e ROS2_BRIDGE_PERF_GOODPUT_TRIAL_SECS="${ROS2_BRIDGE_PERF_GOODPUT_TRIAL_SECS:-}" \
  -e ROS2_BRIDGE_PERF_GOODPUT_RATE_LO="${ROS2_BRIDGE_PERF_GOODPUT_RATE_LO:-}" \
  -e ROS2_BRIDGE_PERF_GOODPUT_RATE_HI="${ROS2_BRIDGE_PERF_GOODPUT_RATE_HI:-}" \
  -e ROS2_BRIDGE_PERF_MSG_LATENCY_SAMPLES="${ROS2_BRIDGE_PERF_MSG_LATENCY_SAMPLES:-}" \
  -e ROS2_BRIDGE_PERF_ONLY="${ROS2_BRIDGE_PERF_ONLY:-}" \
  -e ROS2_RUST_WS="${ROS2_RUST_WS:-/tmp/ros2_rust_ws}" \
  -w "${WS_IN_CONTAINER}" \
  "${CONTAINER}" \
  bash "${WS_IN_CONTAINER}/benches/ros2_bridge_perf/run.sh" --local

echo "==> copying reports back"
docker cp "${CONTAINER}:${WS_IN_CONTAINER}/docs/zh/ros2-bridge-perf-report.md" \
  "${ROOT}/docs/zh/ros2-bridge-perf-report.md"
docker cp "${CONTAINER}:${WS_IN_CONTAINER}/docs/en/ros2-bridge-perf-report.md" \
  "${ROOT}/docs/en/ros2-bridge-perf-report.md"
echo "done: ${ROOT}/docs/zh/ros2-bridge-perf-report.md"
echo "done: ${ROOT}/docs/en/ros2-bridge-perf-report.md"
