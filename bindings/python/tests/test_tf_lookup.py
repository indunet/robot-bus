"""TF buffer / listener smoke (needs native extension from `just python-dev`)."""

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
        "no_discovery": True,
    }


def _static_edge(parent: str, child: str, x: float, y: float):
    from robot_bus.geometry_msgs.msg.v1 import Quaternion, Transform, TransformStamped, Vector3
    from robot_bus.std_msgs.msg.v1 import Header
    from robot_bus.tf2_msgs.msg.v1 import TFMessage

    t = TransformStamped(
        header=Header(frame_id=parent),
        child_frame_id=child,
        transform=Transform(
            translation=Vector3(x=x, y=y, z=0.0),
            rotation=Quaternion(x=0.0, y=0.0, z=0.0, w=1.0),
        ),
    )
    return TFMessage(transforms=[t])


def test_offline_buffer() -> None:
    from robot_bus import TfBuffer

    buf = TfBuffer()
    buf.set_transform_msg(_static_edge("base_link", "camera", 1.0, 0.0), is_static=True)
    assert buf.can_transform("base_link", "camera")
    t = buf.lookup_transform("base_link", "camera")
    assert t.child_frame_id == "camera"
    assert abs(t.transform.translation.x - 1.0) < 1e-9


def test_listener_against_broker() -> None:
    import robot_bus
    from robot_bus import TfListener, TransformBroadcaster
    from robot_bus.tf2_msgs.msg.v1 import TFMessage

    with robot_bus.RobotBusBroker.start(**_ephemeral_binds()) as broker:
        node = robot_bus.Node(
            "py-tf",
            message_xsub=broker.message_xsub_bind,
            message_xpub=broker.message_xpub_bind,
            service_frontend=broker.service_frontend_bind,
            service_backend=broker.service_backend_bind,
            action_frontend=broker.action_frontend_bind,
            action_backend=broker.action_backend_bind,
        )
        listener = TfListener(node)
        buf = listener.buffer()
        br = TransformBroadcaster(node.create_publisher("/tf_static", TFMessage))
        node.start()
        time.sleep(0.2)
        br.send(_static_edge("odom", "base_link", 0.0, 2.0))
        deadline = time.time() + 3.0
        while time.time() < deadline and not buf.can_transform("odom", "base_link"):
            time.sleep(0.02)
        assert buf.can_transform("odom", "base_link")
        t = buf.lookup_transform("odom", "base_link")
        assert abs(t.transform.translation.y - 2.0) < 1e-9
        node.shutdown()
        node.wait()


def main() -> int:
    try:
        import robot_bus
    except ImportError as err:
        print(f"skip: native robot_bus not installed ({err})", file=sys.stderr)
        return 0

    if robot_bus.Node is None or not hasattr(robot_bus, "TfBuffer"):
        print("skip: TfBuffer not available (rebuild extension)", file=sys.stderr)
        return 0

    # Offline buffer works even if broker kwargs differ.
    test_offline_buffer()
    test_listener_against_broker()
    print("ok: test_tf_lookup")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
