#!/usr/bin/env bash
# Build and run ROS 2 perf (shm + udp) inside the `ros2` Docker container,
# writing docs/ros2-perf-report.md at the robot-bus repo root.
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
REPORT_HOST="${ROOT}/docs/ros2-perf-report.md"

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
  local out="$3"
  python3 - "$shm_partial" "$udp_partial" "$out" <<'PY'
import re, sys
from pathlib import Path

shm_path, udp_path, out_path = map(Path, sys.argv[1:4])

def parse(path: Path):
    text = path.read_text()
    env = []
    if m := re.search(r"## 环境\n\n(.*?)\n## ", text, re.S):
        env = [ln for ln in m.group(1).strip().splitlines() if ln.startswith("- ")]
    # New table: 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 | p95 | p99 | mean
    rows = {}
    for m in re.finditer(
        r"\| (message pub/sub|service call|action send_goal) \| (\d+) \| (\d+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \|",
        text,
    ):
        rows[m.group(1)] = {
            "sent": m.group(2),
            "recv": m.group(3),
            "elapsed": m.group(4).strip(),
            "pub": m.group(5).strip(),
            "sub": m.group(6).strip(),
            "delivery": m.group(7).strip(),
        }
    section = ""
    # Prefer the section matching the partial filename.
    want = "shm" if "shm" in path.name else "udp"
    for m in re.finditer(r"(## (?:shm|udp).*\n\n\| 场景.*?)(?=\n## |\Z)", text, re.S):
        block = m.group(1).strip() + "\n"
        if want in block.lower()[:40] or (want == "shm" and "Shared Memory" in block) or (want == "udp" and "UDPv4" in block):
            # Keep only if it has real data rows with digits
            if re.search(r"\| message pub/sub \| \d+", block):
                section = block
                break
    if not section:
        for m in re.finditer(r"(## (?:shm|udp).*\n\n\| 场景.*?)(?=\n## |\Z)", text, re.S):
            block = m.group(1).strip() + "\n"
            if re.search(r"\| message pub/sub \| \d+", block):
                section = block
                break
    return env, rows, section

env_s, rows_s, sec_s = parse(shm_path)
env_u, rows_u, sec_u = parse(udp_path)

def cell_msg_pub(rows):
    r = rows.get("message pub/sub")
    return "—" if not r else f"{r['pub']}/s"

def cell_msg_sub(rows):
    r = rows.get("message pub/sub")
    return "—" if not r else f"{r['sub']}/s ({r['delivery']}% delivered)"

def cell_rpc(rows, scenario):
    r = rows.get(scenario)
    return "—" if not r else f"{r['sub']}/s"

lines = []
lines.append("# ROS 2 性能测试报告\n")
lines.append("由 `benches/ros2_perf/run.sh`（容器内 `ros2_perf_bench`）生成，方法对齐 `docs/perf-report.md`。\n")
lines.append("## 环境\n")
for ln in env_s:
    if ln.startswith("- Mode:"):
        continue
    lines.append(ln)
lines.append("- Modes: **shm** (Fast DDS Shared Memory) + **udp** (Fast DDS UDPv4 only)\n")
lines.append("## 方法\n")
lines.append("- RMW: `rmw_fastrtps_cpp`；传输由 Fast DDS XML 固定为 **SHM** 或 **UDPv4**。")
lines.append("- 单进程多 Node + `MultiThreadedExecutor`（本机回环，非跨机）。")
lines.append("- Payload：64 字节；QoS `KeepLast(2048)` best_effort。")
lines.append("- Message **吞吐（主指标）**：在目标速率下限速发送，**二分搜索**丢包率 ≤ 1% 的最大可持续速率（max goodput）；每档约 1s。")
lines.append("- Message **延迟**：另做限速抽样（发一条等收到再发）。")
lines.append("- Service / action 延迟：每次 call / send_goal 本地计时。")
lines.append("- 指标机器相关，不作为 CI 门槛。\n")
lines.append("## 横比\n")
lines.append("message 为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。\n")
lines.append("| 场景 | shm | udp |")
lines.append("|------|-----|-----|")
lines.append(f"| message 发布 | {cell_msg_pub(rows_s)} | {cell_msg_pub(rows_u)} |")
lines.append(f"| message max goodput | {cell_msg_sub(rows_s)} | {cell_msg_sub(rows_u)} |")
lines.append(f"| service call | {cell_rpc(rows_s, 'service call')} | {cell_rpc(rows_u, 'service call')} |")
lines.append(f"| action send_goal | {cell_rpc(rows_s, 'action send_goal')} | {cell_rpc(rows_u, 'action send_goal')} |")
lines.append("")
lines.append(sec_s if sec_s else "## shm（Fast DDS Shared Memory）\n\n_(missing)_\n")
lines.append("")
lines.append(sec_u if sec_u else "## udp（Fast DDS UDPv4，无 SHM）\n\n_(missing)_\n")
lines.append("## 复现\n")
lines.append("```bash")
lines.append("./benches/ros2_perf/run.sh")
lines.append("ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh")
lines.append("```")
Path(out_path).write_text("\n".join(lines) + "\n")
print(f"wrote {out_path}")
PY
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
  mkdir -p "${ROOT}/docs"
  case "${MODE}" in
    shm)
      run_bench_local "${WS}" shm "${REPORT_HOST}"
      ;;
    udp)
      run_bench_local "${WS}" udp "${REPORT_HOST}"
      ;;
    both)
      shm_p="${PERF_DIR}/_out.shm.partial.md"
      udp_p="${PERF_DIR}/_out.udp.partial.md"
      run_bench_local "${WS}" shm "${shm_p}"
      run_bench_local "${WS}" udp "${udp_p}"
      merge_reports "${shm_p}" "${udp_p}" "${REPORT_HOST}"
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

echo "==> copying report back"
docker cp "${CONTAINER}:${WS_IN_CONTAINER}/docs/ros2-perf-report.md" "${REPORT_HOST}"
echo "done: ${REPORT_HOST}"
