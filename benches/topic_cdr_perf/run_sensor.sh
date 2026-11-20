#!/usr/bin/env bash
# Fixed-rate CDR topic comparison at camera and lidar payload sizes.
# Same process layout as run.sh: one Robot Bus process covers tcp/ipc/inproc,
# and ROS 2 runs SHM and loopback UDPv4 separately.
set -euo pipefail

if [[ $# != 3 ]]; then
  echo "usage: $0 <robot_bus_perf> <ros2_perf_bench> <new-output-directory>" >&2
  exit 2
fi
bus_bin=$(realpath "$1")
ros_bin=$(realpath "$2")
if [[ ! -x "$bus_bin" || ! -x "$ros_bin" || "${ROS_DISTRO:-}" != humble ]]; then
  echo "source Humble and its benchmark overlay, and supply both release binaries" >&2
  exit 2
fi
profiles="$(cd "$(dirname "$0")/../ros2_perf/config" && pwd)"
read -r -a rates <<< "${TOPIC_CDR_PERF_RATES:-10 30}"
read -r -a payloads <<< "${TOPIC_CDR_PERF_PAYLOADS:-262144 921600 6220800}"
history="${TOPIC_CDR_PERF_HISTORY:-8}"
samples="${TOPIC_CDR_PERF_LATENCY_SAMPLES:-100}"
if (( ${#rates[@]} == 0 || ${#payloads[@]} == 0 )); then
  echo "provide at least one rate and one payload size" >&2
  exit 2
fi
mkdir -p "$(dirname "$3")"
mkdir "$3"
cd "$3"
printf '%s\n' "${rates[@]}" > rates.txt
printf '%s\n' "${payloads[@]}" > payloads.txt
printf '%s\n' "$history" > history.txt
printf '%s\n' "$samples" > latency-samples.txt
cp "$profiles/fastdds_shm_sensor.xml" fastdds_shm.xml
cp "$profiles/fastdds_udp_sensor.xml" fastdds_udp.xml
export TZ=Asia/Shanghai
export RMW_IMPLEMENTATION=rmw_fastrtps_cpp ROS_LOCALHOST_ONLY=0 ROS_DOMAIN_ID="${ROS_DOMAIN_ID:-128}"
export RMW_FASTRTPS_PUBLICATION_MODE=SYNCHRONOUS
unset RMW_FASTRTPS_USE_QOS_FROM_XML ROS2_PERF_MERGE
export ROBOT_BUS_PERF_ONLY=cdr-topic ROS2_PERF_ONLY=cdr-topic
export ROBOT_BUS_PERF_MSG_LATENCY_SAMPLES="$samples" ROS2_PERF_MSG_LATENCY_SAMPLES="$samples"
export ROBOT_BUS_PERF_GOODPUT_TRIAL_SECS=3 ROS2_PERF_GOODPUT_TRIAL_SECS=3
export ROBOT_BUS_PERF_GOODPUT_SETTLE_MS=100 ROS2_PERF_GOODPUT_SETTLE_MS=100
export ROBOT_BUS_PERF_GOODPUT_RATE_STEP=1 ROS2_PERF_GOODPUT_RATE_STEP=1
export ROBOT_BUS_PERF_MAX_LOSS_PCT=1 ROS2_PERF_MAX_LOSS_PCT=1
export ROBOT_BUS_PERF_MSG_HWM="$history" ROS2_PERF_MSG_HISTORY="$history"
export ROS2_PERF_EXECUTOR_THREADS=1 ROS2_PERF_REPORT=/dev/null
unset ROBOT_BUS_PERF_GOODPUT_TRIAL_MSGS ROS2_PERF_GOODPUT_TRIAL_MSGS

{
  date -Iseconds
  sha256sum "$bus_bin" "$ros_bin" ./*.xml
  uname -a
  cat /etc/os-release
  grep '^Cpus_allowed_list:' /proc/self/status
  cat /sys/fs/cgroup/{cpuset.cpus.effective,cpu.max,memory.max} 2>/dev/null || true
  df -h /dev/shm
  echo "history=$history latency_samples=$samples"
  echo "payloads=${payloads[*]}"
  echo "rates=${rates[*]}"
  env | LC_ALL=C sort | grep -E '^(ROBOT_BUS_PERF_|ROS2_PERF_|ROS_|RMW_|TZ=)'
} > environment.log

udp_out() {
  awk '$1 == "Udp:" && $2 != "InDatagrams" {print $5}' /proc/net/snmp
}

printf 'round\tpayload\trate\tworkload\tstart\tend\tudp_out\texit\n' > execution.tsv
for round in 1 2 3; do
  case "$round" in
    1) workloads=(bus shm udp) ;;
    2) workloads=(shm udp bus) ;;
    3) workloads=(udp bus shm) ;;
  esac
  for payload in "${payloads[@]}"; do
    for rate in "${rates[@]}"; do
      for workload in "${workloads[@]}"; do
        start=$(date -Iseconds)
        before=$(udp_out)
        status=0
        log="round-$round-p$payload-r$rate-$workload.log"
        export ROBOT_BUS_PERF_PAYLOAD_BYTES="$payload" ROS2_PERF_PAYLOAD_BYTES="$payload"
        if [[ "$workload" == bus ]]; then
          ROBOT_BUS_PERF_GOODPUT_RATE_LO="$rate" ROBOT_BUS_PERF_GOODPUT_RATE_HI="$rate" \
            timeout 180 "$bus_bin" >"$log" 2>&1 || status=$?
        else
          ROS2_PERF_MODE="$workload" FASTRTPS_DEFAULT_PROFILES_FILE="$PWD/fastdds_$workload.xml" \
            ROS2_PERF_GOODPUT_RATE_LO="$rate" ROS2_PERF_GOODPUT_RATE_HI="$rate" \
            timeout 180 "$ros_bin" >"$log" 2>&1 || status=$?
        fi
        printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$round" "$payload" "$rate" "$workload" \
          "$start" "$(date -Iseconds)" "$(($(udp_out) - before))" "$status" >> execution.tsv
        printf 'round=%s payload=%s rate=%s workload=%s exit=%s\n' \
          "$round" "$payload" "$rate" "$workload" "$status"
      done
    done
  done
done
