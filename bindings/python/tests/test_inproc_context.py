"""Same-process inproc requires a shared Context with the embedded broker.

Run after: just python-dev
  .venv/bin/python bindings/python/tests/test_inproc_context.py
"""

from __future__ import annotations

import socket
import sys
import time


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def _inproc_binds() -> dict[str, object]:
    """Ephemeral TCP binds, but keep inproc (tcp_only=False)."""
    return {
        "message_xsub_bind": f"tcp://127.0.0.1:{_free_port()}",
        "message_xpub_bind": f"tcp://127.0.0.1:{_free_port()}",
        "service_frontend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "service_backend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "action_frontend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "action_backend_bind": f"tcp://127.0.0.1:{_free_port()}",
        "grpc_listen": f"127.0.0.1:{_free_port()}",
        "tcp_only": False,
        "no_console": True,
    }


def test_inproc_pubsub_with_shared_context() -> None:
    import robot_bus

    ctx = robot_bus.Context()
    with robot_bus.RobotBusBroker.start(context=ctx, **_inproc_binds()):
        time.sleep(0.15)

        hits: list[bytes] = []
        sub = robot_bus.Node.inproc_with_context(ctx, "inproc-sub")

        def on_msg(_topic: str, payload: bytes) -> None:
            hits.append(bytes(payload))

        sub.create_subscription("/inproc/demo", on_msg)
        # Background Rust thread (PyNode is unsendable — do not spin from a Python thread).
        sub.start()
        time.sleep(0.1)

        pub = robot_bus.Node.inproc_with_context(ctx, "inproc-pub")
        topic = pub.create_publisher("/inproc/demo")
        deadline = time.time() + 5.0
        while not hits and time.time() < deadline:
            topic.publish(b"hello-inproc")
            time.sleep(0.02)

        assert hits, "subscription callback did not fire"
        assert hits[0] == b"hello-inproc"

        sub.shutdown()
        sub.stop()
        sub.wait()


def main() -> int:
    try:
        import robot_bus
    except ImportError as err:
        print(f"skip: native robot_bus not installed ({err})", file=sys.stderr)
        return 0

    if not hasattr(robot_bus, "Context"):
        print("skip: Context not available (rebuild extension)", file=sys.stderr)
        return 0

    test_inproc_pubsub_with_shared_context()
    print("ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
