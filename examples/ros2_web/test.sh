#!/usr/bin/env bash
# Run from any directory inside the prepared image or a sourced ROS environment.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p test-results
{
  date -u +%FT%TZ
  printf 'commit=%s\n' "${ROBOT_BUS_TEST_COMMIT:-$(git rev-parse HEAD 2>/dev/null || echo unknown)}"
  uname -a
  cat /etc/os-release
  python3 --version
  node --version
  cargo --version
  protoc --version
  python3 -m pip show robot-bus protobuf
  printf 'ROS_DISTRO=%s\nRMW_IMPLEMENTATION=%s\nROS_DOMAIN_ID=%s\nROS_LOCALHOST_ONLY=%s\n' \
    "${ROS_DISTRO:-unset}" "${RMW_IMPLEMENTATION:-default}" \
    "${ROS_DOMAIN_ID:-default}" "${ROS_LOCALHOST_ONLY:-default}"
  dpkg-query -W ros-humble-rclpy ros-humble-rmw-fastrtps-cpp 2>/dev/null || true
  sha256sum run.py ros_peer.py web/main.ts tests/bridge.spec.ts \
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
while time.monotonic() < deadline:
    try:
        os.kill(int(sys.argv[1]), 0)
        with urllib.request.urlopen('http://127.0.0.1:8080', timeout=1) as response:
            if response.status == 200:
                break
    except OSError:
        time.sleep(0.1)
else:
    raise SystemExit('demo startup timed out; see test-results/backend.log')
PY
npm test
