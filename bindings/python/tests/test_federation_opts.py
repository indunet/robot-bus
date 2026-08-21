"""Smoke tests for broker federation start options.

Run after: just python-dev
"""

from __future__ import annotations

import sys


def _ephemeral_binds(**extra: object) -> dict[str, object]:
    binds: dict[str, object] = {
        "message_xsub_bind": "tcp://127.0.0.1:0",
        "message_xpub_bind": "tcp://127.0.0.1:0",
        "service_frontend_bind": "tcp://127.0.0.1:0",
        "service_backend_bind": "tcp://127.0.0.1:0",
        "action_frontend_bind": "tcp://127.0.0.1:0",
        "action_backend_bind": "tcp://127.0.0.1:0",
        "api_listen": "127.0.0.1:0",
        "tcp_only": True,
        "no_console": True,
    }
    binds.update(extra)
    return binds


def test_start_rejects_invalid_message_peer() -> None:
    import robot_bus

    try:
        robot_bus.RobotBusBroker.start(
            **_ephemeral_binds(message_peers=["tcp://127.0.0.1:0"])
        )
    except RuntimeError as err:
        assert "invalid message peer" in str(err)
    else:
        raise AssertionError("expected invalid message peer to fail")


def test_start_with_federation_peers() -> None:
    import robot_bus

    with robot_bus.RobotBusBroker.start(
        **_ephemeral_binds(
            broker_id="broker-a",
            message_peers=["tcp://127.0.0.1:16561"],
            service_peers=["broker-b=tcp://127.0.0.1:16562"],
            action_peers=["broker-b=tcp://127.0.0.1:16563"],
        )
    ) as broker:
        assert broker.message_xsub_bind.startswith("tcp://")


def main() -> int:
    try:
        import robot_bus
    except ImportError as err:
        raise SystemExit(f"native robot_bus is required (run: just python-dev): {err}") from err
    if robot_bus.RobotBusBroker is None:
        raise SystemExit("native robot_bus is required (run: just python-dev)")

    test_start_rejects_invalid_message_peer()
    test_start_with_federation_peers()
    print("ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
