#!/usr/bin/env python3
"""Ephemeral-broker smoke for examples/topic_imu (Python).

Starts an in-process broker on the default API port (15570) so
`robot_bus.Node(...)` auto-discover works, then runs listener + talker.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def main() -> int:
    import robot_bus

    listener_py = REPO / "examples/topic_imu/python/listener.py"
    talker_py = REPO / "examples/topic_imu/python/talker.py"
    env = {**os.environ, "PYTHONUNBUFFERED": "1"}

    with tempfile.NamedTemporaryFile(
        mode="w+", encoding="utf-8", suffix=".log", delete=False
    ) as log:
        log_path = Path(log.name)

    try:
        with robot_bus.RobotBusBroker.start(
            api_listen="127.0.0.1:15570",
            tcp_only=True,
            no_console=True,
            advertise_host="127.0.0.1",
        ):
            with log_path.open("w", encoding="utf-8") as logf:
                listener = subprocess.Popen(
                    [sys.executable, "-u", str(listener_py)],
                    cwd=str(REPO),
                    env=env,
                    stdout=logf,
                    stderr=subprocess.STDOUT,
                )
            time.sleep(1.0)
            talker = subprocess.run(
                [sys.executable, "-u", str(talker_py)],
                cwd=str(REPO),
                env=env,
                capture_output=True,
                text=True,
                timeout=30,
            )
            if talker.returncode != 0:
                listener.kill()
                print("talker failed:", talker.stdout, talker.stderr, file=sys.stderr)
                return talker.returncode or 1

            time.sleep(1.2)
            listener.terminate()
            try:
                listener.wait(timeout=5)
            except subprocess.TimeoutExpired:
                listener.kill()
                listener.wait(timeout=5)

        out = log_path.read_text(encoding="utf-8")
    finally:
        log_path.unlink(missing_ok=True)

    if "linear_acceleration.z=" not in out:
        print("listener output missing Imu line:\n", out, file=sys.stderr)
        print("talker stdout:\n", talker.stdout, file=sys.stderr)
        return 1

    print("ok: examples topic_imu smoke")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
