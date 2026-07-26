"""Smoke tests for broker federation start options.

Run after: just python-dev
"""

from __future__ import annotations

import socket
import sys


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def _ephemeral_binds(**extra: object) -> dict[str, object]:
    binds: dict[str, object] = {
        "message_xsub_bind": f"tcp://127.0.0.1:{_free_port()}",
        "message_xpub_bind": f"tcp://127.0.0.1:{_free_port()}",
        "service_frontend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "service_backend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "action_frontend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "action_backend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "grpc_listen": f"127.0.0.1:{_free_port()}",
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

    peer_xpub = f"tcp://127.0.0.1:{_free_port()}"
    peer_svc = f"tcp://127.0.0.1:{_free_port()}"
    peer_act = f"tcp://127.0.0.1:{_free_port()}"
    with robot_bus.RobotBusBroker.start(
        **_ephemeral_binds(
            broker_id="broker-a",
            message_peers=[peer_xpub],
            service_peers=[f"broker-b={peer_svc}"],
            action_peers=[f"broker-b={peer_act}"],
        )
    ) as broker:
        assert broker.message_xsub_bind.startswith("tcp://")


def main() -> int:
    test_start_rejects_invalid_message_peer()
    test_start_with_federation_peers()
    print("ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
