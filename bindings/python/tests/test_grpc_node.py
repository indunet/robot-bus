"""Integration tests for gRPC-mode Node (requires native extension + broker).

Run after: just python-dev
  (or: cd bindings/python && maturin develop --features extension-module,grpc)
  .venv/bin/python bindings/python/tests/test_grpc_node.py
"""

from __future__ import annotations

import socket
import sys
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
        "grpc_listen": f"127.0.0.1:{_free_port()}",
        "tcp_only": True,
        "no_console": True,
    }


def test_grpc_constructors() -> None:
    import robot_bus

    node = robot_bus.Node.grpc("web")
    assert node.name == "web"

    node2 = robot_bus.Node.grpc_at("web2", "http://10.0.0.1:15770")
    assert node2.name == "web2"

    node3 = robot_bus.Node("web3", transport="grpc", grpc_url="http://127.0.0.1:15770")
    assert node3.name == "web3"


def test_grpc_node_rejects_publisher_and_servers() -> None:
    import robot_bus

    node = robot_bus.Node.grpc("only-client")
    try:
        node.create_publisher("/t")
    except RuntimeError as err:
        assert "not supported" in str(err)
    else:
        raise AssertionError("expected create_publisher to fail")

    try:
        node.create_service("/svc", lambda _body: b"")
    except RuntimeError as err:
        assert "not supported" in str(err)
    else:
        raise AssertionError("expected create_service to fail")


def test_grpc_node_subscribe_and_service() -> None:
    import robot_bus

    with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
        grpc_url = f"http://{broker.grpc_listen}"

        pub = robot_bus.Publisher(broker.message_xsub_bind)
        server = robot_bus.Node(
            "svc_server",
            message_xsub=broker.message_xsub_bind,
            message_xpub=broker.message_xpub_bind,
            service_frontend=broker.service_frontend_bind,
            service_backend=broker.service_backend_bind,
        )
        server.create_service("svc.py_grpc_echo", lambda body: b"echo:" + bytes(body))
        # Background Rust thread (PyNode is unsendable — do not spin from a Python thread).
        server.start()
        time.sleep(0.2)

        got: list[tuple[str, bytes]] = []
        client = robot_bus.Node.grpc_at("grpc_client", grpc_url)

        def on_msg(topic: str, payload: bytes) -> None:
            got.append((topic, bytes(payload)))

        client.create_subscription("py.grpc.topic", on_msg)
        time.sleep(0.3)

        pub.publish("py.grpc.topic", b"hello-py-grpc")
        deadline = time.time() + 5.0
        while not got and time.time() < deadline:
            client.spin_once(0.05)

        assert got, "subscription callback did not fire"
        assert got[0][0] == "py.grpc.topic"
        assert got[0][1] == b"hello-py-grpc"

        svc = client.create_client("svc.py_grpc_echo")
        reply = svc.call(b"ping", timeout=3.0)
        assert bytes(reply) == b"echo:ping"

        server.shutdown()
        server.stop()
        server.wait()


def test_grpc_node_action_client() -> None:
    import robot_bus

    with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
        grpc_url = f"http://{broker.grpc_listen}"

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

        server.create_action_server("act.py_grpc_demo", handler)
        server.start()
        time.sleep(0.2)

        client = robot_bus.Node.grpc_at("grpc_action", grpc_url)
        action = client.create_action_client("act.py_grpc_demo")
        messages = action.send_goal(b"fly", timeout=5.0)
        assert len(messages) == 3
        assert messages[0]["kind"] == "FEEDBACK"
        assert bytes(messages[0]["body"]) == b"step-1"
        assert messages[2]["kind"] == "RESULT"
        assert bytes(messages[2]["body"]) == b"done:fly"

        server.shutdown()
        server.stop()
        server.wait()


def main() -> int:
    try:
        import robot_bus  # noqa: F401
    except ImportError as err:
        print(f"skip: native robot_bus not installed ({err})", file=sys.stderr)
        return 0

    if not hasattr(robot_bus.Node, "grpc"):
        print("skip: Node.grpc not available (rebuild with --features grpc)", file=sys.stderr)
        return 0

    test_grpc_constructors()
    test_grpc_node_rejects_publisher_and_servers()
    test_grpc_node_subscribe_and_service()
    test_grpc_node_action_client()
    print("ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
