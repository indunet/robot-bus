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


def test_inproc_action_goal_handle() -> None:
    import robot_bus

    ctx = robot_bus.Context()
    with robot_bus.RobotBusBroker.start(context=ctx, **_inproc_binds()):
        server = robot_bus.Node.inproc_with_context(ctx, "inproc-action-server")

        def on_goal(body: bytes):
            return [
                ("FEEDBACK", b"step:" + bytes(body)),
                ("RESULT", b"done:" + bytes(body)),
            ]

        server.create_action_server("/inproc/action", on_goal)
        server.start()
        time.sleep(0.1)

        client = robot_bus.Node.inproc_with_context(
            ctx, "inproc-action-client"
        )
        action = client.create_action_client("/inproc/action")
        feedback: list[bytes] = []
        goal = action.send_goal(
            b"move",
            feedback_callback=lambda body: feedback.append(bytes(body)),
        )

        assert goal.action_name == "/inproc/action"
        assert goal.goal_id
        assert bytes(goal.result(timeout=3.0)) == b"done:move"
        assert feedback == [b"step:move"]

        server.shutdown()
        server.stop()
        server.wait()


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
    test_inproc_action_goal_handle()
    print("ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
