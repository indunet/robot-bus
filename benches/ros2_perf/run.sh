#!/usr/bin/env bash
# Build and run ROS 2 perf (shm + udp) inside the `ros2` Docker container,
# writing docs/zh/ros2-perf-report.md and docs/en/ros2-perf-report.md.
set -euo pipefail

PERF_DIR="$(cd "$(dirname "$0")" && pwd)"
# Host: benches/ros2_perf → repo root. Container --local: flat ros2_perf under WS → WS parent.
if [[ "$(basename "$(dirname "${PERF_DIR}")")" == "benches" ]]; then
  ROOT="$(cd "${PERF_DIR}/../.." && pwd)"
else
  ROOT="$(cd "${PERF_DIR}/.." && pwd)"
fi
CONTAINER="${ROS2_PERF_CONTAINER:-ros2}"
WS_IN_CONTAINER="${ROS2_PERF_WS:-/tmp/ros2_perf_ws}"
REPORT_HOST_ZH="${ROOT}/docs/zh/ros2-perf-report.md"
REPORT_HOST_EN="${ROOT}/docs/en/ros2-perf-report.md"
# Back-compat alias used by single-mode paths / docker layout under WS.
REPORT_HOST="${REPORT_HOST_ZH}"

# Optional overrides (useful for smoke tests):
#   ROS2_PERF_GOODPUT_TRIAL_MSGS / ROS2_PERF_GOODPUT_RATE_LO / ROS2_PERF_GOODPUT_RATE_HI
#   ROS2_PERF_MAX_LOSS_PCT / ROS2_PERF_MSG_LATENCY_SAMPLES
#   ROS2_PERF_SVC_ITERS / ROS2_PERF_ACT_ITERS
SVC_ITERS="${ROS2_PERF_SVC_ITERS:-100000}"
ACT_ITERS="${ROS2_PERF_ACT_ITERS:-100000}"
MSG_LATENCY_SAMPLES="${ROS2_PERF_MSG_LATENCY_SAMPLES:-5000}"
GOODPUT_TRIAL_MSGS="${ROS2_PERF_GOODPUT_TRIAL_MSGS:-}"
GOODPUT_TRIAL_SECS="${ROS2_PERF_GOODPUT_TRIAL_SECS:-1.0}"
GOODPUT_RATE_LO="${ROS2_PERF_GOODPUT_RATE_LO:-500}"
GOODPUT_RATE_HI="${ROS2_PERF_GOODPUT_RATE_HI:-500000}"
MAX_LOSS_PCT="${ROS2_PERF_MAX_LOSS_PCT:-1.0}"
ONLY="${ROS2_PERF_ONLY:-}"

usage() {
  cat <<'EOF'
Usage: benches/ros2_perf/run.sh [--local] [--build-only] [--mode shm|udp|both]

  (default)  Sync sources into Docker container `ros2`, colcon build, run shm+udp, pull report.
  --local    Assume already inside the container (or ROS sourced); build+run in synced ros2_perf.
  --build-only  Only build, do not run benches.
  --mode     shm | udp | both (default: both)
EOF
}

LOCAL=0
BUILD_ONLY=0
MODE=both
while [[ $# -gt 0 ]]; do
  case "$1" in
    --local) LOCAL=1; shift ;;
    --build-only) BUILD_ONLY=1; shift ;;
    --mode) MODE="${2:-both}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

merge_reports() {
  local shm_partial="$1"
  local udp_partial="$2"
  local out_zh="$3"
  local out_en="$4"
  python3 "${PERF_DIR}/merge_reports.py" "$shm_partial" "$udp_partial" "$out_zh" "$out_en"
}

run_bench_local() {
  local ws="$1"
  local mode="$2"
  local report_partial="$3"
  source_ros "${ws}/install/setup.bash"

  export RMW_IMPLEMENTATION=rmw_fastrtps_cpp
  export ROS_LOCALHOST_ONLY=1
  export ROS2_PERF_MODE="${mode}"
  export ROS2_PERF_SVC_ITERS="${SVC_ITERS}"
  export ROS2_PERF_ACT_ITERS="${ACT_ITERS}"
  export ROS2_PERF_MSG_LATENCY_SAMPLES="${MSG_LATENCY_SAMPLES}"
  export ROS2_PERF_GOODPUT_TRIAL_MSGS="${GOODPUT_TRIAL_MSGS}"
  export ROS2_PERF_GOODPUT_TRIAL_SECS="${GOODPUT_TRIAL_SECS}"
  export ROS2_PERF_GOODPUT_RATE_LO="${GOODPUT_RATE_LO}"
  export ROS2_PERF_GOODPUT_RATE_HI="${GOODPUT_RATE_HI}"
  export ROS2_PERF_MAX_LOSS_PCT="${MAX_LOSS_PCT}"
  if [[ -n "${ONLY}" ]]; then
    export ROS2_PERF_ONLY="${ONLY}"
  fi
  export ROS2_PERF_REPORT="${report_partial}"
  export ROS2_PERF_MERGE=0
  unset FASTRTPS_DEFAULT_PROFILES_FILE
  case "${mode}" in
    shm)
      export FASTRTPS_DEFAULT_PROFILES_FILE="${PERF_DIR}/config/fastdds_shm.xml"
      ;;
    udp)
      export FASTRTPS_DEFAULT_PROFILES_FILE="${PERF_DIR}/config/fastdds_udp.xml"
      ;;
    *)
      echo "bad mode: ${mode}" >&2
      exit 1
      ;;
  esac

  echo "==> running mode=${mode} profile=${FASTRTPS_DEFAULT_PROFILES_FILE}"
  ros2 run ros2_perf ros2_perf_bench
}

source_ros() {
  # ROS setup scripts reference optional vars; don't trip `set -u`.
  set +u
  # shellcheck disable=SC1091
  source /opt/ros/humble/setup.bash
  if [[ $# -ge 1 && -f "$1" ]]; then
    # shellcheck disable=SC1091
    source "$1"
  fi
  set -u
}

build_local() {
  local ws="$1"
  mkdir -p "${ws}/src"
  rm -rf "${ws}/src/ros2_perf"
  cp -a "${PERF_DIR}/src/ros2_perf" "${ws}/src/ros2_perf"
  source_ros
  cd "${ws}"
  colcon build --packages-select ros2_perf --cmake-args -DCMAKE_BUILD_TYPE=Release
}

if [[ "${LOCAL}" -eq 1 ]]; then
  WS="${PERF_DIR}/_ws"
  build_local "${WS}"
  if [[ "${BUILD_ONLY}" -eq 1 ]]; then
    exit 0
  fi
  mkdir -p "${ROOT}/docs/zh" "${ROOT}/docs/en"
  case "${MODE}" in
    shm)
      run_bench_local "${WS}" shm "${REPORT_HOST_ZH}"
      ;;
    udp)
      run_bench_local "${WS}" udp "${REPORT_HOST_ZH}"
      ;;
    both)
      shm_p="${PERF_DIR}/_out.shm.partial.md"
      udp_p="${PERF_DIR}/_out.udp.partial.md"
      run_bench_local "${WS}" shm "${shm_p}"
      run_bench_local "${WS}" udp "${udp_p}"
      merge_reports "${shm_p}" "${udp_p}" "${REPORT_HOST_ZH}" "${REPORT_HOST_EN}"
      rm -f "${shm_p}" "${udp_p}"
      ;;
    *)
      echo "bad --mode ${MODE}" >&2
      exit 1
      ;;
  esac
  exit 0
fi

# Host path: sync into container and run there.
if ! docker inspect "${CONTAINER}" >/dev/null 2>&1; then
  echo "Docker container '${CONTAINER}' not found." >&2
  exit 1
fi
if [[ "$(docker inspect -f '{{.State.Running}}' "${CONTAINER}")" != "true" ]]; then
  echo "Docker container '${CONTAINER}' is not running." >&2
  exit 1
fi

echo "==> syncing ${PERF_DIR} -> ${CONTAINER}:${WS_IN_CONTAINER}"
docker exec "${CONTAINER}" mkdir -p "${WS_IN_CONTAINER}"
docker cp "${PERF_DIR}/." "${CONTAINER}:${WS_IN_CONTAINER}/ros2_perf_src"
# Normalize layout expected by scripts inside container
docker exec "${CONTAINER}" bash -lc "rm -rf '${WS_IN_CONTAINER}/ros2_perf' && mkdir -p '${WS_IN_CONTAINER}' && cp -a '${WS_IN_CONTAINER}/ros2_perf_src' '${WS_IN_CONTAINER}/ros2_perf'"

echo "==> build + run inside ${CONTAINER}"
mode_args=(--local --mode "${MODE}")
if [[ "${BUILD_ONLY}" -eq 1 ]]; then
  mode_args+=(--build-only)
fi
# Pass args safely into the container shell.
docker exec \
  -e ROS2_PERF_SVC_ITERS="${SVC_ITERS}" \
  -e ROS2_PERF_ACT_ITERS="${ACT_ITERS}" \
  -e ROS2_PERF_MSG_LATENCY_SAMPLES="${MSG_LATENCY_SAMPLES}" \
  -e ROS2_PERF_GOODPUT_TRIAL_MSGS="${GOODPUT_TRIAL_MSGS}" \
  -e ROS2_PERF_GOODPUT_TRIAL_SECS="${GOODPUT_TRIAL_SECS}" \
  -e ROS2_PERF_GOODPUT_RATE_LO="${GOODPUT_RATE_LO}" \
  -e ROS2_PERF_GOODPUT_RATE_HI="${GOODPUT_RATE_HI}" \
  -e ROS2_PERF_MAX_LOSS_PCT="${MAX_LOSS_PCT}" \
  -e ROS2_PERF_ONLY="${ONLY}" \
  -w "${WS_IN_CONTAINER}/ros2_perf" \
  "${CONTAINER}" \
  bash ./run.sh "${mode_args[@]}"

echo "==> copying reports back"
docker exec "${CONTAINER}" mkdir -p "${WS_IN_CONTAINER}/docs/zh" "${WS_IN_CONTAINER}/docs/en"
docker cp "${CONTAINER}:${WS_IN_CONTAINER}/docs/zh/ros2-perf-report.md" "${REPORT_HOST_ZH}"
docker cp "${CONTAINER}:${WS_IN_CONTAINER}/docs/en/ros2-perf-report.md" "${REPORT_HOST_EN}"
echo "done: ${REPORT_HOST_ZH}"
echo "done: ${REPORT_HOST_EN}"
