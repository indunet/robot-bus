"""Minimal unit tests for Python Ros2Bridge builder (no ROS spin required)."""

from __future__ import annotations

try:
    import pytest
except ImportError:  # pragma: no cover - CI uses pytest

    class pytest:  # type: ignore[no-redef]
        @staticmethod
        def raises(exc, match=None):
            class _Ctx:
                def __enter__(self):
                    return self

                def __exit__(self, t, v, tb):
                    if t is None:
                        raise AssertionError("did not raise")
                    if not issubclass(t, exc):
                        return False
                    if match and match not in str(v):
                        raise AssertionError(f"{v!r} does not contain {match!r}")
                    return True

            return _Ctx()

        @staticmethod
        def importorskip(name):
            try:
                __import__(name)
            except ImportError as err:
                print(f"skip {name}: {err}")
                raise SystemExit(0) from err


def test_builder_requires_mapper():
    from robot_bus.ros2_bridge import Ros2Bridge

    with pytest.raises(ValueError, match="mapper"):
        Ros2Bridge.new("t").route("/a", "/a").add()


def test_builder_accepts_concrete_mappers_without_build():
    # Importing mappers pulls ROS message packages; skip if unsourced.
    pytest.importorskip("std_msgs")
    pytest.importorskip("std_srvs")
    from robot_bus.ros2_bridge import (
        Ros2Bridge,
        StdMsgsStringMapper,
        TriggerServiceMapper,
    )

    b = (
        Ros2Bridge.new("t")
        .route("/chatter", "/chatter")
        .mapper(StdMsgsStringMapper())
        .add()
        .service("/reset", "/reset")
        .mapper(TriggerServiceMapper())
        .timeout(2.0)
        .add()
    )
    assert len(b._routes) == 1
    assert len(b._services) == 1


def test_lazy_defaults_off():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    class Dummy:
        def type_name(self) -> str:
            return "test/msg/Dummy"

        def ros_msg_type(self):
            return object

        def ros_to_bus(self, _msg) -> bytes:
            return b""

        def bus_to_ros(self, _payload: bytes):
            return None

    b = Ros2Bridge.new("t").route("/a", "/a").mapper(Dummy()).add()
    assert b._routes[0]["lazy"] is False
    assert b._routes[0]["direction"] == Direction.Ros2ToBus


def test_lazy_opt_in_ros2_to_bus():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    class Dummy:
        def type_name(self) -> str:
            return "test/msg/Dummy"

        def ros_msg_type(self):
            return object

        def ros_to_bus(self, _msg) -> bytes:
            return b""

        def bus_to_ros(self, _payload: bytes):
            return None

    b = Ros2Bridge.new("t").route("/cam", "/cam").mapper(Dummy()).lazy().add()
    assert b._routes[0]["lazy"] is True
    assert b._routes[0]["direction"] == Direction.Ros2ToBus


def test_lazy_rejects_bus_to_ros2():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    class Dummy:
        def type_name(self) -> str:
            return "test/msg/Dummy"

        def ros_msg_type(self):
            return object

        def ros_to_bus(self, _msg) -> bytes:
            return b""

        def bus_to_ros(self, _payload: bytes):
            return None

    with pytest.raises(ValueError, match="lazy"):
        (
            Ros2Bridge.new("t")
            .route("/a", "/a")
            .mapper(Dummy())
            .direction(Direction.BusToRos2)
            .lazy()
            .add()
        )


def test_lazy_rejects_attach_only_mapper():
    from robot_bus.ros2_bridge import Ros2Bridge

    class AttachOnly:
        def type_name(self) -> str:
            return "test/msg/Dummy"

        def attach(self, ctx) -> None:
            raise AssertionError("should not attach during add()")

    with pytest.raises(ValueError, match="lazy"):
        Ros2Bridge.new("t").route("/a", "/a").mapper(AttachOnly()).lazy().add()


def test_lazy_and_eager_routes_independent():
    from robot_bus.ros2_bridge import Ros2Bridge

    class Dummy:
        def type_name(self) -> str:
            return "test/msg/Dummy"

        def ros_msg_type(self):
            return object

        def ros_to_bus(self, _msg) -> bytes:
            return b""

        def bus_to_ros(self, _payload: bytes):
            return None

    b = (
        Ros2Bridge.new("t")
        .route("/a", "/a")
        .mapper(Dummy())
        .add()
        .route("/b", "/b")
        .mapper(Dummy())
        .lazy()
        .add()
    )
    assert b._routes[0]["lazy"] is False
    assert b._routes[1]["lazy"] is True


def test_ros2_available_checks_rclpy():
    import robot_bus

    try:
        import rclpy  # noqa: F401
    except ImportError:
        assert robot_bus.ros2_available() is False
    else:
        assert robot_bus.ros2_available() is True


if __name__ == "__main__":
    test_builder_requires_mapper()
    test_lazy_defaults_off()
    test_lazy_opt_in_ros2_to_bus()
    test_lazy_rejects_bus_to_ros2()
    test_lazy_rejects_attach_only_mapper()
    test_lazy_and_eager_routes_independent()
    test_ros2_available_checks_rclpy()
    try:
        test_builder_accepts_concrete_mappers_without_build()
    except SystemExit:
        pass
    print("test_ros2_bridge_builder ok")
