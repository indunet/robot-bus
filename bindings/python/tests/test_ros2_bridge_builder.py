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
    assert TopicQos.default().console_label() == "keep_last(10).reliable"
    assert TopicQos.latched().console_label() == "keep_last(1).reliable.transient_local"


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


def test_route_health_idle_and_warn_rate_limit():
    from robot_bus.ros2_bridge.builder.drop_stats import RouteHealth

    health = RouteHealth()
    assert health.take_idle_event(True, False) is False
    assert health.take_idle_event(True, True) is True
    assert health.take_idle_event(True, True) is False
    health.record_rx()
    health.record_tx()
    assert health.is_idle(True, True) is False
    assert health.should_log_warn() is True
    assert health.should_log_warn() is False
    assert health.rx == 1
    assert health.tx == 1


def test_forward_with_health_counts_route_and_bridge():
    from robot_bus.ros2_bridge.builder.drop_stats import (
        DropStats,
        RouteHealth,
        forward_ros_to_bus,
    )

    stats = DropStats()
    health = RouteHealth()

    def boom(_msg) -> bytes:
        raise ValueError("bad ros")

    forward_ros_to_bus(stats, "/cam", boom, lambda _p: None, object(), health)
    assert stats.snapshot()["convert_fail"] == 1
    assert health.convert_fail == 1
    assert health.rx == 1
    assert health.tx == 0


def test_bridge_snapshot_proto_roundtrip():
    from robot_bus.robot_bus_interfaces.msg.v1 import BridgeSnapshot

    snap = BridgeSnapshot()
    snap.bridge_id = "b1"
    snap.bridge_name = "cam_bridge"
    row = snap.routes.add()
    row.kind = "topic"
    row.direction = "ros→bus"
    row.ros_name = "/cam"
    row.bus_name = "/cam"
    row.convert_fail = 2
    row.idle = True
    out = BridgeSnapshot()
    out.ParseFromString(snap.SerializeToString())
    assert out.bridge_id == "b1"
    assert out.bridge_name == "cam_bridge"
    assert len(out.routes) == 1
    assert out.routes[0].ros_name == "/cam"
    assert out.routes[0].convert_fail == 2
    assert out.routes[0].idle is True


def test_ros2_bridge_drop_stats_starts_zero():
    from robot_bus.ros2_bridge import Ros2Bridge

    bridge = Ros2Bridge()
    assert bridge.drop_stats() == {"convert_fail": 0, "decode_fail": 0, "publish_fail": 0}


class _FakeBus:
    def __init__(self, on_spin=None):
        self.spins = 0
        self.shutdowns = 0
        self._on_spin = on_spin

    def spin_once(self, _timeout):
        self.spins += 1
        if self._on_spin is not None:
            self._on_spin(self.spins)

    def shutdown(self):
        self.shutdowns += 1


class _FakeContext:
    def __init__(self):
        self.alive = True

    def ok(self):
        return self.alive


class _FakeRosNode:
    def __init__(self):
        self.context = _FakeContext()
        self.destroyed = 0

    def destroy_node(self):
        self.destroyed += 1


def _bare_bridge(on_spin=None):
    from robot_bus.ros2_bridge import Ros2Bridge

    bridge = Ros2Bridge()
    bridge._bus = _FakeBus(on_spin)
    bridge._ros_node = _FakeRosNode()
    return bridge


def test_shutdown_idempotent_and_closes_bus():
    bridge = _bare_bridge()
    bus, ros = bridge._bus, bridge._ros_node
    bridge.shutdown()
    bridge.shutdown()
    bridge.close()
    assert bus.shutdowns == 1
    assert ros.destroyed == 1
    with pytest.raises(RuntimeError, match="shut down"):
        bridge.spin_once(0.0)


def test_context_manager_shuts_down():
    bridge = _bare_bridge()
    bus = bridge._bus
    with bridge as entered:
        assert entered is bridge
        bridge.spin_once(0.0)
    assert bus.shutdowns == 1


def test_run_stops_on_ros_shutdown():
    holder = {}

    def on_spin(n):
        if n == 3:
            holder["bridge"]._ros_node.context.alive = False

    bridge = _bare_bridge(on_spin)
    holder["bridge"] = bridge
    bus = bridge._bus
    bridge.run()
    assert bus.spins == 3
    assert bus.shutdowns == 1


def test_run_treats_ctrl_c_as_stop():
    def on_spin(_n):
        raise KeyboardInterrupt

    bridge = _bare_bridge(on_spin)
    bus = bridge._bus
    bridge.run()
    assert bus.shutdowns == 1


def test_run_reraises_executor_failure_and_shuts_down():
    def on_spin(_n):
        raise RuntimeError("ROS executor failed: boom")

    bridge = _bare_bridge(on_spin)
    bus = bridge._bus
    with pytest.raises(RuntimeError, match="boom"):
        bridge.run()
    assert bus.shutdowns == 1


def test_spin_once_tolerates_idle_buses_without_busy_looping():
    import time

    def idle(_n):
        raise RuntimeError("nothing registered; connect_subscriber, register worker, or create_timer first")

    bridge = _bare_bridge(idle)
    bridge._rpc_bus = _FakeBus(idle)
    t0 = time.monotonic()
    bridge.spin_once(0.05)  # polling is bounded to 10 ms
    assert time.monotonic() - t0 >= 0.009
    assert bridge._bus.spins == 1 and bridge._rpc_bus.spins == 1

    def broken(_n):
        raise RuntimeError("socket closed")

    bridge._rpc_bus = _FakeBus(broken)
    with pytest.raises(RuntimeError, match="socket closed"):
        bridge.spin_once(0.0)


def test_executor_error_after_ros_shutdown_is_not_reported():
    class Boom:
        def spin(self):
            raise RuntimeError("context is not valid")

    bridge = _bare_bridge()
    bridge._executor = Boom()
    bridge._ros_node.context.alive = False  # what rclpy's SIGINT handler does
    bridge._ros_spin()
    assert bridge._executor_error is None

    bridge = _bare_bridge()
    bridge._executor = Boom()
    bridge._ros_spin()  # context still valid: a real failure
    assert "not valid" in str(bridge._executor_error)


def test_mapper_type_name_from_ros_interface():
    from robot_bus.ros2_bridge.builder.config import mapper_type_name

    Twist = type("Twist", (), {"__module__": "geometry_msgs.msg._twist"})
    AddTwoInts = type("AddTwoInts", (), {"__module__": "example_interfaces.srv._add_two_ints"})

    class TopicMapper:
        def ros_msg_type(self):
            return Twist

    class ServiceMapper:
        def ros_srv_type(self):
            return AddTwoInts

    class Named(TopicMapper):
        def type_name(self):
            return "my_pkg/msg/Custom"

    assert mapper_type_name(TopicMapper()) == "geometry_msgs/msg/Twist"
    assert mapper_type_name(ServiceMapper()) == "example_interfaces/srv/AddTwoInts"
    assert mapper_type_name(Named()) == "my_pkg/msg/Custom"
    assert mapper_type_name(Dummy()) == "object"


def _builder():
    from robot_bus.ros2_bridge import Ros2Bridge

    return Ros2Bridge.new("t")


def test_mapper_by_type_name_for_every_kind():
    from robot_bus.ros2_bridge import (
        FibonacciActionMapper,
        GeometryMsgsTwistMapper,
        SensorMsgsImuMapper,
        StdMsgsColorRgbaMapper,
        TriggerServiceMapper,
    )

    b = (
        _builder()
        .from_ros("/imu", _qos_ros()).to_bus("/imu", _qos_bus())
        .mapper("sensor_msgs/msg/Imu").add()
        .from_bus("/cmd_vel", _qos_bus()).to_ros("/cmd_vel", _qos_ros())
        .mapper("geometry_msgs/Twist").add()
        .from_ros("/color", _qos_ros()).to_bus("/color", _qos_bus())
        .mapper("std_msgs/msg/ColorRGBA").add()
        .service().from_bus("/reset", _qos_bus()).to_ros("/reset", _qos_ros())
        .mapper("std_srvs/srv/Trigger").add()
        .action().from_bus("/fib", _qos_bus()).to_ros("/fib", _qos_ros())
        .mapper("example_interfaces/action/Fibonacci").add()
    )
    kinds = [type(r["mapper"]) for r in b._routes]
    assert kinds == [SensorMsgsImuMapper, GeometryMsgsTwistMapper, StdMsgsColorRgbaMapper]
    assert isinstance(b._services[0]["mapper"], TriggerServiceMapper)
    assert isinstance(b._actions[0]["mapper"], FibonacciActionMapper)


def test_every_builtin_mapper_has_a_type_name():
    from robot_bus.ros2_bridge import mappers, registry

    covered = {cls for _, cls in registry._load_builtin().values()}
    assert [e for e in mappers.__all__ if getattr(mappers, e) not in covered] == []


def test_mapper_type_name_errors_are_actionable():
    b = _builder().from_ros("/x", _qos_ros()).to_bus("/x", _qos_bus())
    with pytest.raises(ValueError, match="sensor_msgs/msg/Imu"):
        b.mapper("sensor_msgs/msg/Imuu")
    with pytest.raises(ValueError, match="register_mapper"):
        b.mapper("my_pkg/msg/Nope")
    with pytest.raises(ValueError, match="use it on .service()"):
        b.mapper("std_srvs/srv/Trigger")
    with pytest.raises(ValueError, match="pkg/msg/Name"):
        b.mapper("Twist")


def test_register_custom_mapper():
    from robot_bus.ros2_bridge import register_mapper, registry

    try:
        @register_mapper("my_pkg/msg/Sum")
        class SumMapper(Dummy):
            pass

        shared = Dummy()
        register_mapper("my_pkg/srv/AddTwoInts", shared)

        b = (
            _builder()
            .from_ros("/sum", _qos_ros()).to_bus("/sum", _qos_bus()).mapper("my_pkg/msg/Sum").add()
            .service().from_ros("/add", _qos_ros()).to_bus("/add", _qos_bus())
            .mapper("my_pkg/srv/AddTwoInts").add()
        )
        assert isinstance(b._routes[0]["mapper"], SumMapper)
        assert b._services[0]["mapper"] is shared

        register_mapper("my_pkg/msg/Sum", SumMapper)  # same mapper again is fine
        with pytest.raises(ValueError, match="override=True"):
            register_mapper("my_pkg/msg/Sum", Dummy)
        with pytest.raises(ValueError, match="override=True"):
            register_mapper("std_msgs/msg/String", Dummy)
        register_mapper("std_msgs/msg/String", Dummy, override=True)
        b = _builder().from_ros("/s", _qos_ros()).to_bus("/s", _qos_bus()).mapper("std_msgs/msg/String")
        assert type(b._mapper) is Dummy
    finally:
        registry._user.clear()


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
    test_route_health_idle_and_warn_rate_limit()
    test_forward_with_health_counts_route_and_bridge()
    test_bridge_snapshot_proto_roundtrip()
    test_ros2_bridge_drop_stats_starts_zero()
    test_shutdown_idempotent_and_closes_bus()
    test_context_manager_shuts_down()
    test_run_stops_on_ros_shutdown()
    test_run_treats_ctrl_c_as_stop()
    test_run_reraises_executor_failure_and_shuts_down()
    test_spin_once_tolerates_idle_buses_without_busy_looping()
    test_executor_error_after_ros_shutdown_is_not_reported()
    test_mapper_type_name_from_ros_interface()
    test_mapper_by_type_name_for_every_kind()
    test_every_builtin_mapper_has_a_type_name()
    test_mapper_type_name_errors_are_actionable()
    test_register_custom_mapper()
    test_ros2_available_checks_rclpy()
    try:
        test_builder_accepts_concrete_mappers_without_build()
    except SystemExit:
        pass
    print("test_ros2_bridge_builder ok")
