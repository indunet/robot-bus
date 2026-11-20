#!/usr/bin/env bash
# Run from any directory inside the prepared image or a sourced ROS environment.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p test-results
{
  date -u +%FT%TZ
  # Keep thread-pool sizing comparable to hosted CI on large local machines.
  test_cpus=$(python3 -c 'import os; print(",".join(map(str, sorted(os.sched_getaffinity(0))[:4])))')
  taskset -pc "$test_cpus" "$$"
  printf 'commit=%s\n' "${ROBOT_BUS_TEST_COMMIT:-$(git rev-parse HEAD 2>/dev/null || echo unknown)}"
  uname -a
  cat /etc/os-release
  python3 --version
  node --version
  cargo --version
  protoc --version
  python3 -m pip show robot-bus protobuf
  printf 'ROS_DISTRO=%s\nEXPECTED_ROS_DISTRO=%s\nRMW_IMPLEMENTATION=%s\nROS_DOMAIN_ID=%s\nROS_LOCALHOST_ONLY=%s\n' \
    "${ROS_DISTRO:-unset}" "${ROBOT_BUS_TEST_ROS_DISTRO:-${ROS_DISTRO:-unset}}" "${RMW_IMPLEMENTATION:-default}" \
    "${ROS_DOMAIN_ID:-default}" "${ROS_LOCALHOST_ONLY:-default}"
  if [[ -z "${ROS_DISTRO:-}" || "$ROS_DISTRO" != "${ROBOT_BUS_TEST_ROS_DISTRO:-$ROS_DISTRO}" ]]; then
    echo "unexpected ROS environment; source the requested ROS distribution before running"
    exit 1
  fi
  dpkg-query -W "ros-$ROS_DISTRO-rclpy" "ros-$ROS_DISTRO-rmw-fastrtps-cpp" \
    "ros-$ROS_DISTRO-example-interfaces" 2>/dev/null || true
  python3 - <<'PY'
import rclpy
from rclpy.utilities import get_rmw_implementation_identifier

print(f"rclpy_path={rclpy.__file__}")
print(f"rmw_identifier={get_rmw_implementation_identifier()}")
PY
  sha256sum test.sh run.py ros_peer.py web/main.ts tests/bridge.spec.ts \
    ../../bindings/python/robot_bus/ros2_bridge/builder/bridge.py
} > test-results/environment.log 2>&1
# Refuse to test another application's existing listener.
python3 - <<'PY'
import socket
for port in (8080, 15560):
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', port))
PY
python3 -u run.py > test-results/backend.log 2>&1 &
backend_pid=$!
cleanup() {
  kill -TERM "$backend_pid" 2>/dev/null || true
  # Bound cleanup even if a native shutdown regresses.
  (sleep 15; kill -KILL "$backend_pid" 2>/dev/null || true) &
  watchdog=$!
  wait "$backend_pid" 2>/dev/null || true
  kill "$watchdog" 2>/dev/null || true
}
trap cleanup EXIT
python3 - "$backend_pid" <<'PY'
import os, sys, time, urllib.request
deadline = time.monotonic() + 30
request = urllib.request.Request('http://127.0.0.1:8080', method='HEAD')
while time.monotonic() < deadline:
    try:
        os.kill(int(sys.argv[1]), 0)
        with urllib.request.urlopen(request, timeout=1) as response:
            if response.status == 200:
                break
    except OSError:
        time.sleep(0.1)
else:
    raise SystemExit('demo startup timed out; see test-results/backend.log')
PY
npm test
