"""Integration tests for WebSocket RPC Node (requires native extension + broker).

Run after: just python-dev
  (or: cd bindings/python && maturin develop --features extension-module,ws)
  .venv/bin/python bindings/python/tests/test_ws_node.py
"""

from __future__ import annotations

import socket
import time


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
        "no_console": True,
    }


def _require_native():
    try:
        import robot_bus
    except ImportError as err:
        raise SystemExit(f"native robot_bus is required (run: just python-dev): {err}") from err
    if robot_bus.Node is None or not hasattr(robot_bus, "RobotBusBroker"):
        raise SystemExit("native robot_bus is required (run: just python-dev)")
    if not hasattr(robot_bus.Node, "ws"):
        raise SystemExit("Node.ws is required (rebuild with --features ws)")
    return robot_bus


def test_ws_constructors() -> None:
    import robot_bus

    node = robot_bus.Node.ws("web")
    assert node.name == "web"

    node2 = robot_bus.Node.ws_at("web2", "http://10.0.0.1:15570")
    assert node2.name == "web2"

    node3 = robot_bus.Node("web3", transport="ws", ws_url="http://127.0.0.1:15570")
    assert node3.name == "web3"


def test_ws_node_rejects_servers() -> None:
    import robot_bus

    node = robot_bus.Node.ws("only-client")
    try:
        node.create_service("/svc", lambda _body: b"")
    except RuntimeError as err:
        assert "not supported" in str(err)
    else:
        raise AssertionError("expected create_service to fail")


def test_ws_node_publish() -> None:
    import robot_bus

    with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
        ws_url = f"http://{broker.api_listen}"
        sub = robot_bus.Subscriber(broker.message_xpub_bind)
        sub.subscribe("py.ws.pub")
        time.sleep(0.2)

        node = robot_bus.Node.ws_at("ws_pub", ws_url)
        pub = node.create_publisher("py.ws.pub")
        pub.publish(b"from-py-ws")

        topic, payload = sub.receive(timeout=3.0)
        assert topic == "py.ws.pub"
        assert bytes(payload) == b"from-py-ws"

def test_ws_node_subscribe_and_service() -> None:
    import robot_bus

    with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
        ws_url = f"http://{broker.api_listen}"

        pub = robot_bus.Publisher(broker.message_xsub_bind)
        server = robot_bus.Node(
            "svc_server",
            message_xsub=broker.message_xsub_bind,
            message_xpub=broker.message_xpub_bind,
            service_frontend=broker.service_frontend_bind,
            service_backend=broker.service_backend_bind,
        )
        server.create_service("svc.py_ws_echo", lambda body: b"echo:" + bytes(body))
        # Background Rust thread (PyNode is unsendable — do not spin from a Python thread).
        server.start()
        time.sleep(0.2)

        got: list[tuple[str, bytes]] = []
        client = robot_bus.Node.ws_at("ws_client", ws_url)

        def on_msg(topic: str, payload: bytes) -> None:
            got.append((topic, bytes(payload)))

        client.create_subscription("py.ws.topic", on_msg)
        time.sleep(0.3)

        pub.publish("py.ws.topic", b"hello-py-ws")
        deadline = time.time() + 5.0
        while not got and time.time() < deadline:
            client.spin_once(0.05)

        assert got, "subscription callback did not fire"
        assert got[0][0] == "py.ws.topic"
        assert got[0][1] == b"hello-py-ws"

        svc = client.create_client("svc.py_ws_echo")
        reply = svc.call(b"ping", timeout=3.0)
        assert bytes(reply) == b"echo:ping"

        server.shutdown()
        server.stop()
        server.wait()


def test_ws_node_action_client() -> None:
    import robot_bus

    with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
        ws_url = f"http://{broker.api_listen}"

        server = robot_bus.Node(
            "act_server",
            action_frontend=broker.action_frontend_bind,
            action_backend=broker.action_backend_bind,
        )

        def handler(body: bytes):
            return [
                ("FEEDBACK", b"step-1"),
                ("FEEDBACK", b"step-2"),
                ("RESULT", b"done:" + bytes(body)),
            ]

        server.create_action_server("act.py_ws_demo", handler)
        server.start()
        time.sleep(0.2)

        client = robot_bus.Node.ws_at("ws_action", ws_url)
        action = client.create_action_client("act.py_ws_demo")
        feedback: list[bytes] = []
        goal = action.send_goal(
            b"fly",
            goal_id="py-ws-goal",
            timeout=5.0,
            feedback_callback=lambda body: feedback.append(bytes(body)),
        )
        assert goal.goal_id == "py-ws-goal"
        assert goal.action_name == "act.py_ws_demo"
        assert bytes(goal.result(timeout=5.0)) == b"done:fly"
        assert feedback == [b"step-1", b"step-2"]

        server.shutdown()
        server.stop()
        server.wait()


def main() -> int:
    _require_native()
    test_ws_constructors()
    test_ws_node_rejects_servers()
    test_ws_node_publish()
    test_ws_node_subscribe_and_service()
    test_ws_node_action_client()
    print("ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
