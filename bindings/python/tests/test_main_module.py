"""Standalone broker CLI is ``python -m robot_bus.broker`` (no console-script binary).

Run after: just python-dev
  python3 bindings/python/tests/test_main_module.py
"""

from __future__ import annotations

import subprocess
import sys


def test_help() -> None:
    proc = subprocess.run(
        [sys.executable, "-m", "robot_bus.broker", "--help"],
        check=False,
        capture_output=True,
        text=True,
    )
    out = (proc.stdout or "") + (proc.stderr or "")
    if proc.returncode != 0:
        raise SystemExit(f"python -m robot_bus.broker --help exited {proc.returncode}: {out}")
    if "python -m robot_bus.broker" not in out:
        raise SystemExit(f"help text missing python -m robot_bus.broker:\n{out}")
    print("python -m robot_bus.broker --help ok")


def test_package_main_points_at_broker() -> None:
    proc = subprocess.run(
        [sys.executable, "-m", "robot_bus"],
        check=False,
        capture_output=True,
        text=True,
    )
    out = (proc.stdout or "") + (proc.stderr or "")
    if proc.returncode == 0:
        raise SystemExit(f"python -m robot_bus should not start the broker:\n{out}")
    if "python -m robot_bus.broker" not in out:
        raise SystemExit(f"python -m robot_bus should point at broker module:\n{out}")
    print("python -m robot_bus points at robot_bus.broker ok")


if __name__ == "__main__":
    test_help()
    test_package_main_points_at_broker()
