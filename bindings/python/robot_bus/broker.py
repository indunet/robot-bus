"""Standalone broker: ``python -m robot_bus.broker [options]``.

Same flags as the Rust ``cargo run --bin robot_bus_broker`` / C++ ``robot_bus_broker`` CLI.
Prefer ``RobotBusBroker.start()`` in application code; this module is for demos
and a long-running process.
"""

from __future__ import annotations

from robot_bus import run_broker


def main() -> None:
    if run_broker is None:
        raise SystemExit(
            "robot_bus native extension is not installed "
            "(pip install robot-bus, or just python-dev)"
        )
    run_broker()


if __name__ == "__main__":
    main()
