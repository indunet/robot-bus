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


def _qos_ros():
    from robot_bus.ros2_bridge import TopicQos

    return TopicQos.keep_last(10).reliable()


def _qos_bus():
    from robot_bus.ros2_bridge import TopicQos

    return TopicQos.keep_last(8).best_effort()


class Dummy:
    def ros_msg_type(self):
        return object

    def ros_to_bus(self, _msg) -> bytes:
        return b""

    def bus_to_ros(self, _payload: bytes):
        return None


def test_topic_qos_presets():
    from robot_bus.ros2_bridge import TopicQos

    assert TopicQos.sensor_data() == TopicQos.keep_last(5).best_effort()
    assert TopicQos.default() == TopicQos.keep_last(10).reliable()
    assert TopicQos.latched() == TopicQos.keep_last(1).reliable().transient_local()
    assert TopicQos.bus() == TopicQos.keep_last(8).best_effort()
    assert TopicQos.sensor_data().is_best_effort is True
    assert TopicQos.default().is_reliable is True
    assert TopicQos.latched().is_transient_local is True
    assert TopicQos.bus().is_best_effort is True
    assert TopicQos.bus().depth == 8


def test_topic_qos_requires_reliability():
    from robot_bus.ros2_bridge import TopicQos

    q = TopicQos.keep_last(10).reliable()
    assert q.depth == 10
    assert q.is_reliable is True
    assert q.is_volatile is True
    assert q.is_transient_local is False
    be = TopicQos.keep_last(5).best_effort()
    assert be.is_best_effort is True
    latched = TopicQos.keep_last(1).reliable().transient_local()
    assert latched.is_transient_local is True
    assert latched.is_volatile is False
    assert latched.is_reliable is True
    assert latched.depth == 1


def test_incomplete_topic_qos_rejected():
    from robot_bus.ros2_bridge import Ros2Bridge, TopicQos

    with pytest.raises(TypeError, match="TopicQos"):
        Ros2Bridge.new("t").from_ros("/a", TopicQos.keep_last(10))


def test_builder_accepts_concrete_mappers_without_build():
    pytest.importorskip("std_msgs")
    pytest.importorskip("std_srvs")
    from robot_bus.ros2_bridge import (
        Ros2Bridge,
        StdMsgsStringMapper,
        TriggerServiceMapper,
    )

    b = (
        Ros2Bridge.new("t")
        .from_ros("/chatter", _qos_ros())
        .to_bus("/chatter", _qos_bus())
        .mapper(StdMsgsStringMapper())
        .add()
        .service()
        .from_ros("/reset", _qos_ros())
        .to_bus("/reset", _qos_bus())
        .mapper(TriggerServiceMapper())
        .timeout(2.0)
        .add()
    )
    assert len(b._routes) == 1
    assert len(b._services) == 1


def test_lazy_defaults_off():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    b = (
        Ros2Bridge.new("t")
        .from_ros("/a", _qos_ros())
        .to_bus("/a", _qos_bus())
        .mapper(Dummy())
        .add()
    )
    assert b._routes[0]["lazy"] is False
    assert b._routes[0]["direction"] == Direction.Ros2ToBus


def test_lazy_opt_in_ros2_to_bus():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    b = (
        Ros2Bridge.new("t")
        .from_ros("/cam", _qos_ros())
        .to_bus("/cam", _qos_bus())
        .mapper(Dummy())
        .lazy()
        .add()
    )
    assert b._routes[0]["lazy"] is True
    assert b._routes[0]["direction"] == Direction.Ros2ToBus


def test_from_bus_to_ros():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    b = (
        Ros2Bridge.new("t")
        .from_bus("/a", _qos_bus())
        .to_ros("/a", _qos_ros())
        .mapper(Dummy())
        .add()
    )
    assert b._routes[0]["direction"] == Direction.BusToRos2
    assert b._routes[0]["lazy"] is False


def test_lazy_rejects_attach_only_mapper():
    from robot_bus.ros2_bridge import Ros2Bridge

    class AttachOnly:
        def attach(self, ctx) -> None:
            raise AssertionError("should not attach during add()")

    with pytest.raises(ValueError, match="lazy"):
        (
            Ros2Bridge.new("t")
            .from_ros("/a", _qos_ros())
            .to_bus("/a", _qos_bus())
            .mapper(AttachOnly())
            .lazy()
            .add()
        )


def test_lazy_and_eager_routes_independent():
    from robot_bus.ros2_bridge import Ros2Bridge

    b = (
        Ros2Bridge.new("t")
        .from_ros("/a", _qos_ros())
        .to_bus("/a", _qos_bus())
        .mapper(Dummy())
        .add()
        .from_ros("/b", _qos_ros())
        .to_bus("/b", _qos_bus())
        .mapper(Dummy())
        .lazy()
        .add()
    )
    assert b._routes[0]["lazy"] is False
    assert b._routes[1]["lazy"] is True


def test_qos_stored_per_endpoint():
    from robot_bus.ros2_bridge import Ros2Bridge, TopicQos

    ros = TopicQos.keep_last(20).best_effort()
    bus = TopicQos.keep_last(4).best_effort()
    b = Ros2Bridge.new("t").from_ros("/a", ros).to_bus("/a", bus).mapper(Dummy()).add()
    assert b._routes[0]["ros_qos"] == ros
    assert b._routes[0]["bus_qos"] == bus

    latched = TopicQos.keep_last(1).reliable().transient_local()
    b2 = (
        Ros2Bridge.new("t")
        .from_ros("/tf_static", latched)
        .to_bus("/tf_static", bus)
        .mapper(Dummy())
        .add()
    )
    assert b2._routes[0]["ros_qos"] == latched
    assert b2._routes[0]["ros_qos"].is_transient_local is True

    latched = TopicQos.keep_last(1).reliable().transient_local()
    b2 = (
        Ros2Bridge.new("t")
        .from_ros("/tf_static", latched)
        .to_bus("/tf_static", bus)
        .mapper(Dummy())
        .add()
    )
    assert b2._routes[0]["ros_qos"] == latched
    assert b2._routes[0]["ros_qos"].is_transient_local is True


def test_bus_reliable_rejected():
    from robot_bus.ros2_bridge import Ros2Bridge, TopicQos

    with pytest.raises(ValueError, match="best_effort"):
        (
            Ros2Bridge.new("t")
            .from_ros("/a", _qos_ros())
            .to_bus("/a", TopicQos.keep_last(8).reliable())
            .mapper(Dummy())
            .add()
        )


def test_service_from_bus_to_ros():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    class DummyService:
        def type_name(self) -> str:
            return "test/srv/Dummy"

    b = (
        Ros2Bridge.new("t")
        .service()
        .from_bus("/a", _qos_bus())
        .to_ros("/a", _qos_ros())
        .mapper(DummyService())
        .timeout(1.0)
        .add()
    )
    assert b._services[0]["direction"] == Direction.BusToRos2
    assert b._services[0]["timeout"] == 1.0
    assert b._services[0]["ros_qos"] == _qos_ros()
    assert b._services[0]["bus_qos"] == _qos_bus()


def test_action_stores_ros_qos():
    from robot_bus.ros2_bridge import Direction, Ros2Bridge

    class DummyAction:
        def type_name(self) -> str:
            return "test/action/Dummy"

    b = (
        Ros2Bridge.new("t")
        .action()
        .from_ros("/f", _qos_ros())
        .to_bus("/f", _qos_bus())
        .mapper(DummyAction())
        .add()
    )
    assert b._actions[0]["direction"] == Direction.Ros2ToBus
    assert b._actions[0]["ros_qos"] == _qos_ros()
    assert b._actions[0]["bus_qos"] == _qos_bus()


def test_service_qos_required_on_ros_endpoint():
    from robot_bus.ros2_bridge import Ros2Bridge, TopicQos

    class DummyService:
        def type_name(self) -> str:
            return "test/srv/Dummy"

    with pytest.raises(TypeError, match="TopicQos"):
        (
            Ros2Bridge.new("t")
            .service()
            .from_ros("/a", TopicQos.keep_last(10))
            .to_bus("/a", _qos_bus())
            .mapper(DummyService())
            .add()
        )


def test_service_qos_required_on_bus_endpoint():
    from robot_bus.ros2_bridge import Ros2Bridge, TopicQos

    class DummyService:
        def type_name(self) -> str:
            return "test/srv/Dummy"

    with pytest.raises(TypeError, match="TopicQos"):
        (
            Ros2Bridge.new("t")
            .service()
            .from_ros("/a", _qos_ros())
            .to_bus("/a", TopicQos.keep_last(8))
            .mapper(DummyService())
            .add()
        )

    with pytest.raises(ValueError, match="best_effort"):
        (
            Ros2Bridge.new("t")
            .service()
            .from_ros("/a", _qos_ros())
            .to_bus("/a", TopicQos.keep_last(8).reliable())
            .mapper(DummyService())
            .add()
        )


def test_drop_stats_helper_counts_convert_decode_publish():
    from google.protobuf.message import DecodeError

    from robot_bus.ros2_bridge.builder.drop_stats import (
        DropStats,
        forward_bus_to_ros,
        forward_ros_to_bus,
    )

    stats = DropStats()
    assert stats.snapshot() == {"convert_fail": 0, "decode_fail": 0, "publish_fail": 0}

    published: list[bytes] = []

    def boom_convert(_msg) -> bytes:
        raise ValueError("bad ros")

    forward_ros_to_bus(stats, "/cam", boom_convert, published.append, object())
    assert stats.snapshot()["convert_fail"] == 1
    assert published == []

    def boom_publish(_payload: bytes) -> None:
        raise RuntimeError("bus down")

    forward_ros_to_bus(stats, "/cam", lambda _m: b"ok", boom_publish, object())
    assert stats.snapshot()["publish_fail"] == 1

    def boom_decode(_payload: bytes):
        raise DecodeError("truncated")

    forward_bus_to_ros(stats, "/cam", boom_decode, lambda _m: None, b"xx")
    assert stats.snapshot()["decode_fail"] == 1

    def boom_bus_convert(_payload: bytes):
        raise TypeError("mapper")

    forward_bus_to_ros(stats, "/cam", boom_bus_convert, lambda _m: None, b"xx")
    snap = stats.snapshot()
    assert snap == {"convert_fail": 2, "decode_fail": 1, "publish_fail": 1}


def test_ros2_bridge_drop_stats_starts_zero():
    from robot_bus.ros2_bridge import Ros2Bridge

    bridge = Ros2Bridge()
    assert bridge.drop_stats() == {"convert_fail": 0, "decode_fail": 0, "publish_fail": 0}


def test_ros2_available_checks_rclpy():
    import robot_bus

    try:
        import rclpy  # noqa: F401
    except ImportError:
        assert robot_bus.ros2_available() is False
    else:
        assert robot_bus.ros2_available() is True


if __name__ == "__main__":
    test_topic_qos_presets()
    test_topic_qos_requires_reliability()
    test_incomplete_topic_qos_rejected()
    test_lazy_defaults_off()
    test_lazy_opt_in_ros2_to_bus()
    test_from_bus_to_ros()
    test_lazy_rejects_attach_only_mapper()
    test_lazy_and_eager_routes_independent()
    test_qos_stored_per_endpoint()
    test_bus_reliable_rejected()
    test_service_from_bus_to_ros()
    test_service_qos_required_on_ros_endpoint()
    test_service_qos_required_on_bus_endpoint()
    test_action_stores_ros_qos()
    test_drop_stats_helper_counts_convert_decode_publish()
    test_ros2_bridge_drop_stats_starts_zero()
    test_ros2_available_checks_rclpy()
    try:
        test_builder_accepts_concrete_mappers_without_build()
    except SystemExit:
        pass
    print("test_ros2_bridge_builder ok")
