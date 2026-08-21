"""Embedded Web console is compiled into the Python wheel.

Run after: just python-dev
  .venv/bin/python bindings/python/tests/test_console.py
"""

from __future__ import annotations

import socket
import time
import urllib.error
import urllib.request


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def _ephemeral_binds() -> dict[str, object]:
    return {
        "message_xsub_bind": f"tcp://127.0.0.1:{_free_port()}",
        "message_xpub_bind": f"tcp://127.0.0.1:{_free_port()}",
        "service_frontend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "service_backend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "action_frontend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "action_backend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "api_listen": f"127.0.0.1:{_free_port()}",
        "tcp_only": True,
    }


def _require_native():
    try:
        import robot_bus
    except ImportError as err:
        raise SystemExit(f"native robot_bus is required (run: just python-dev): {err}") from err
    if robot_bus.Node is None or not hasattr(robot_bus, "RobotBusBroker"):
        raise SystemExit("native robot_bus is required (run: just python-dev)")
    if not hasattr(robot_bus.RobotBusBroker, "console_listen"):
        raise SystemExit("Web console is required (rebuild with --features console)")
    return robot_bus


def test_console_index_and_status() -> None:
    import robot_bus

    with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
        listen = broker.console_listen
        assert listen, "console_listen should be set when the console feature is on"
        time.sleep(0.2)
        base = f"http://{listen}"

        with urllib.request.urlopen(f"{base}/", timeout=3) as resp:
            html = resp.read()
            assert resp.status == 200
        assert b"<html" in html.lower() or b"<!doctype" in html.lower(), html[:200]

        with urllib.request.urlopen(f"{base}/api/v1/status", timeout=3) as resp:
            status = resp.read().decode()
            assert resp.status == 200
        assert "ONLINE" in status or "status" in status.lower(), status


def test_no_console_skips_ui() -> None:
    import robot_bus

    binds = _ephemeral_binds()
    binds["no_console"] = True
    with robot_bus.RobotBusBroker.start(**binds) as broker:
        assert broker.console_listen is None
        url = f"http://{broker.api_listen}/"
        try:
            with urllib.request.urlopen(url, timeout=3) as resp:
                body = resp.read()
        except urllib.error.HTTPError as err:
            assert err.code == 404
            return
        assert b"<html" not in body.lower()
        assert b"<!doctype" not in body.lower()


def main() -> int:
    _require_native()
    test_console_index_and_status()
    test_no_console_skips_ui()
    print("ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
