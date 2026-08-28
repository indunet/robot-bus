"""Fluent `Ros2Bridge` builder (rclpy + robot_bus.Node)."""

from __future__ import annotations

import threading
import time
from enum import IntEnum
from typing import Any, Callable, Optional

import robot_bus

SERVICE_CALL_TIMEOUT = 5.0
ACTION_CALL_TIMEOUT = 30.0
CONSOLE_DETECT_TIMEOUT = 2.0
TOPIC_DEMAND = "/robot_bus/topic_demand"
TOPICS_SNAPSHOT = "/robot_bus/topics"


class Direction(IntEnum):
    Ros2ToBus = 0
    BusToRos2 = 1


class TopicQosKeepLast:
    def __init__(self, depth: int) -> None:
        self._depth = int(depth)

    def reliable(self) -> "TopicQos":
        return TopicQos(self._depth, False)

    def best_effort(self) -> "TopicQos":
        return TopicQos(self._depth, True)


class TopicQos:
    """KeepLast depth plus reliability.

    Same type on ROS endpoints for topics, services, and actions.
    Bus only uses it on topics (must be ``.best_effort()``); service / action
    bus names have no QoS.
    """

    def __init__(self, depth: int, best_effort: bool) -> None:
        self._depth = int(depth)
        self._best_effort = bool(best_effort)

    @staticmethod
    def keep_last(depth: int) -> TopicQosKeepLast:
        return TopicQosKeepLast(depth)

    @property
    def depth(self) -> int:
        return self._depth

    @property
    def is_best_effort(self) -> bool:
        return self._best_effort

    @property
    def is_reliable(self) -> bool:
        return not self._best_effort

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, TopicQos):
            return NotImplemented
        return self._depth == other._depth and self._best_effort == other._best_effort


def should_enable_ros_subscription(
    lazy: bool, console_live: Optional[bool], subscribers: int
) -> bool:
    if not lazy:
        return True
    if console_live is None:
        return False
    if console_live is False:
        return True
    return subscribers > 0


def _ros_qos(qos: TopicQos) -> Any:
    from rclpy.qos import (
        HistoryPolicy,
        QoSProfile,
        ReliabilityPolicy,
    )

    profile = QoSProfile(
        history=HistoryPolicy.KEEP_LAST,
        depth=max(int(qos.depth), 0),
    )
    profile.reliability = (
        ReliabilityPolicy.BEST_EFFORT if qos.is_best_effort else ReliabilityPolicy.RELIABLE
    )
    return profile


def _ros_service_qos(qos: TopicQos) -> Any:
    from rclpy.qos import (
        HistoryPolicy,
        QoSProfile,
        ReliabilityPolicy,
        qos_profile_services_default,
    )

    base = qos_profile_services_default
    return QoSProfile(
        history=HistoryPolicy.KEEP_LAST,
        depth=max(int(qos.depth), 0),
        reliability=(
            ReliabilityPolicy.BEST_EFFORT if qos.is_best_effort else ReliabilityPolicy.RELIABLE
        ),
        durability=base.durability,
        lifespan=base.lifespan,
        deadline=base.deadline,
        liveliness=base.liveliness,
        liveliness_lease_duration=base.liveliness_lease_duration,
        avoid_ros_namespace_conventions=base.avoid_ros_namespace_conventions,
    )


def _require_topic_qos(qos: Any, who: str) -> TopicQos:
    if not isinstance(qos, TopicQos):
        raise TypeError(
            f"{who} requires TopicQos.keep_last(n).reliable() or .best_effort()"
        )
    return qos


def _require_bus_best_effort(qos: TopicQos) -> None:
    if qos.is_reliable:
        raise ValueError(
            "ros2 bridge: bus TopicQos must be .best_effort() "
            "(PUB/SUB has no reliable delivery)"
        )


def _topic_supports_lazy(mapper: Any) -> bool:
    return callable(getattr(mapper, "ros_msg_type", None)) and callable(
        getattr(mapper, "ros_to_bus", None)
    )


class TopicWireContext:
    def __init__(
        self,
        ros_node: Any,
        bus_node: Any,
        ros_topic: str,
        bus_topic: str,
        direction: Direction,
        keep_alive: list,
        qos: Any = 10,
        bus_qos_depth: Optional[int] = None,
    ) -> None:
        self.ros_node = ros_node
        self.bus_node = bus_node
        self.ros_topic = ros_topic
        self.bus_topic = bus_topic
        self.direction = direction
        self.keep_alive = keep_alive
        self.qos = qos
        self.bus_qos_depth = bus_qos_depth

    def retain(self, obj: Any) -> None:
        self.keep_alive.append(obj)


class ServiceWireContext:
    def __init__(
        self,
        ros_node: Any,
        bus_node: Any,
        ros_service: str,
        bus_service: str,
        direction: Direction,
        timeout_secs: float,
        callback_group: Any,
        keep_alive: list,
        ros_qos: Any = None,
    ) -> None:
        self.ros_node = ros_node
        self.bus_node = bus_node
        self.ros_service = ros_service
        self.bus_service = bus_service
        self.direction = direction
        self.timeout_secs = timeout_secs
        self.callback_group = callback_group
        self.keep_alive = keep_alive
        self.ros_qos = ros_qos

    def retain(self, obj: Any) -> None:
        self.keep_alive.append(obj)


class ActionWireContext:
    def __init__(
        self,
        ros_node: Any,
        bus_node: Any,
        ros_action: str,
        bus_action: str,
        direction: Direction,
        timeout_secs: float,
        callback_group: Any,
        keep_alive: list,
        ros_qos: Any = None,
    ) -> None:
        self.ros_node = ros_node
        self.bus_node = bus_node
        self.ros_action = ros_action
        self.bus_action = bus_action
        self.direction = direction
        self.timeout_secs = timeout_secs
        self.callback_group = callback_group
        self.keep_alive = keep_alive
        self.ros_qos = ros_qos

    def retain(self, obj: Any) -> None:
        self.keep_alive.append(obj)


class Ros2BridgeBuilder:
    def __init__(self, name: str) -> None:
        self._name = name
        self._bus_factory: Callable[[str], Any] = lambda n: robot_bus.Node.tcp(n, "localhost")
        self._routes: list[dict[str, Any]] = []
        self._services: list[dict[str, Any]] = []
        self._actions: list[dict[str, Any]] = []

    def bus_tcp(self, host: str = "localhost") -> "Ros2BridgeBuilder":
        self._bus_factory = lambda n, h=host: robot_bus.Node.tcp(n, h)
        return self

    def bus_ipc(self) -> "Ros2BridgeBuilder":
        self._bus_factory = lambda n: robot_bus.Node.ipc(n)
        return self

    def bus_ipc_at(self, dir: str) -> "Ros2BridgeBuilder":
        self._bus_factory = lambda n, d=dir: robot_bus.Node.ipc(n, d)
        return self

    def bus_discover(
        self, api_url: str = "", timeout: float = 0.0, broker_id: str = ""
    ) -> "Ros2BridgeBuilder":
        kwargs: dict[str, Any] = {}
        if api_url:
            kwargs["api_url"] = api_url
        if broker_id:
            kwargs["broker_id"] = broker_id
        if timeout > 0:
            kwargs["timeout"] = timeout

        def factory(name: str, kw: dict[str, Any] = kwargs) -> Any:
            return robot_bus.Node.discover(name, "tcp", **kw)

        self._bus_factory = factory
        return self

    def from_ros(self, ros_topic: str, ros_qos: TopicQos) -> "FromRos":
        return FromRos(self, ros_topic, _require_topic_qos(ros_qos, "from_ros"))

    def from_bus(self, bus_topic: str, bus_qos: TopicQos) -> "FromBus":
        qos = _require_topic_qos(bus_qos, "from_bus")
        _require_bus_best_effort(qos)
        return FromBus(self, bus_topic, qos)

    def service(self) -> "Service":
        return Service(self)

    def action(self) -> "Action":
        return Action(self)

    def build(self) -> "Ros2Bridge":
        if not self._routes and not self._services and not self._actions:
            raise ValueError(
                "Ros2Bridge requires at least one topic route, service, or action"
            )
        return Ros2Bridge._from_builder(self)


class FromRos:
    def __init__(self, parent: Ros2BridgeBuilder, ros_topic: str, ros_qos: TopicQos) -> None:
        self._parent = parent
        self._ros_topic = ros_topic
        self._ros_qos = ros_qos

    def to_bus(self, bus_topic: str, bus_qos: TopicQos) -> "FromRosToBus":
        qos = _require_topic_qos(bus_qos, "to_bus")
        _require_bus_best_effort(qos)
        return FromRosToBus(self._parent, self._ros_topic, self._ros_qos, bus_topic, qos)


class FromRosToBus:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_topic: str,
        ros_qos: TopicQos,
        bus_topic: str,
        bus_qos: TopicQos,
    ) -> None:
        self._parent = parent
        self._ros_topic = ros_topic
        self._ros_qos = ros_qos
        self._bus_topic = bus_topic
        self._bus_qos = bus_qos

    def mapper(self, mapper: Any) -> "Ros2ToBusReady":
        if mapper is None:
            raise ValueError("ros2 bridge route: mapper must not be None")
        return Ros2ToBusReady(
            self._parent,
            self._ros_topic,
            self._ros_qos,
            self._bus_topic,
            self._bus_qos,
            mapper,
        )


class Ros2ToBusReady:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_topic: str,
        ros_qos: TopicQos,
        bus_topic: str,
        bus_qos: TopicQos,
        mapper: Any,
    ) -> None:
        self._parent = parent
        self._ros_topic = ros_topic
        self._ros_qos = ros_qos
        self._bus_topic = bus_topic
        self._bus_qos = bus_qos
        self._mapper = mapper
        self._lazy = False

    def lazy(self) -> "Ros2ToBusReady":
        self._lazy = True
        return self

    def add(self) -> Ros2BridgeBuilder:
        if self._lazy and not _topic_supports_lazy(self._mapper):
            raise ValueError(
                "ros2 bridge route: .lazy() is not supported for this custom TopicMapper "
                "(attach-only); implement ros_msg_type/ros_to_bus"
            )
        self._parent._routes.append(
            {
                "ros_topic": self._ros_topic,
                "bus_topic": self._bus_topic,
                "mapper": self._mapper,
                "direction": Direction.Ros2ToBus,
                "lazy": self._lazy,
                "ros_qos": self._ros_qos,
                "bus_qos": self._bus_qos,
            }
        )
        return self._parent


class FromBus:
    def __init__(self, parent: Ros2BridgeBuilder, bus_topic: str, bus_qos: TopicQos) -> None:
        self._parent = parent
        self._bus_topic = bus_topic
        self._bus_qos = bus_qos

    def to_ros(self, ros_topic: str, ros_qos: TopicQos) -> "FromBusToRos":
        return FromBusToRos(
            self._parent,
            ros_topic,
            _require_topic_qos(ros_qos, "to_ros"),
            self._bus_topic,
            self._bus_qos,
        )


class FromBusToRos:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_topic: str,
        ros_qos: TopicQos,
        bus_topic: str,
        bus_qos: TopicQos,
    ) -> None:
        self._parent = parent
        self._ros_topic = ros_topic
        self._ros_qos = ros_qos
        self._bus_topic = bus_topic
        self._bus_qos = bus_qos

    def mapper(self, mapper: Any) -> "BusToRos2Ready":
        if mapper is None:
            raise ValueError("ros2 bridge route: mapper must not be None")
        return BusToRos2Ready(
            self._parent,
            self._ros_topic,
            self._ros_qos,
            self._bus_topic,
            self._bus_qos,
            mapper,
        )


class BusToRos2Ready:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_topic: str,
        ros_qos: TopicQos,
        bus_topic: str,
        bus_qos: TopicQos,
        mapper: Any,
    ) -> None:
        self._parent = parent
        self._ros_topic = ros_topic
        self._ros_qos = ros_qos
        self._bus_topic = bus_topic
        self._bus_qos = bus_qos
        self._mapper = mapper

    def add(self) -> Ros2BridgeBuilder:
        self._parent._routes.append(
            {
                "ros_topic": self._ros_topic,
                "bus_topic": self._bus_topic,
                "mapper": self._mapper,
                "direction": Direction.BusToRos2,
                "lazy": False,
                "ros_qos": self._ros_qos,
                "bus_qos": self._bus_qos,
            }
        )
        return self._parent


class Service:
    def __init__(self, parent: Ros2BridgeBuilder) -> None:
        self._parent = parent

    def from_ros(self, ros_service: str, ros_qos: TopicQos) -> "ServiceFromRos":
        return ServiceFromRos(self._parent, ros_service, _require_topic_qos(ros_qos, "from_ros"))

    def from_bus(self, bus_service: str) -> "ServiceFromBus":
        return ServiceFromBus(self._parent, bus_service)


class ServiceFromRos:
    def __init__(self, parent: Ros2BridgeBuilder, ros_service: str, ros_qos: TopicQos) -> None:
        self._parent = parent
        self._ros_service = ros_service
        self._ros_qos = ros_qos

    def to_bus(self, bus_service: str) -> "ServicePair":
        return ServicePair(
            self._parent, self._ros_service, bus_service, self._ros_qos, Direction.Ros2ToBus
        )


class ServiceFromBus:
    def __init__(self, parent: Ros2BridgeBuilder, bus_service: str) -> None:
        self._parent = parent
        self._bus_service = bus_service

    def to_ros(self, ros_service: str, ros_qos: TopicQos) -> "ServicePair":
        return ServicePair(
            self._parent,
            ros_service,
            self._bus_service,
            _require_topic_qos(ros_qos, "to_ros"),
            Direction.BusToRos2,
        )


class ServicePair:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_service: str,
        bus_service: str,
        ros_qos: TopicQos,
        direction: Direction,
    ) -> None:
        self._parent = parent
        self._ros_service = ros_service
        self._bus_service = bus_service
        self._ros_qos = ros_qos
        self._direction = direction

    def mapper(self, mapper: Any) -> "ServiceReady":
        return ServiceReady(
            self._parent,
            self._ros_service,
            self._bus_service,
            mapper,
            self._direction,
            self._ros_qos,
        )


class ServiceReady:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_service: str,
        bus_service: str,
        mapper: Any,
        direction: Direction,
        ros_qos: TopicQos,
    ) -> None:
        self._parent = parent
        self._ros_service = ros_service
        self._bus_service = bus_service
        self._mapper = mapper
        self._direction = direction
        self._ros_qos = ros_qos
        self._timeout = SERVICE_CALL_TIMEOUT

    def timeout(self, timeout_secs: float) -> "ServiceReady":
        self._timeout = timeout_secs
        return self

    def add(self) -> Ros2BridgeBuilder:
        self._parent._services.append(
            {
                "ros_service": self._ros_service,
                "bus_service": self._bus_service,
                "mapper": self._mapper,
                "direction": self._direction,
                "timeout": self._timeout,
                "ros_qos": self._ros_qos,
            }
        )
        return self._parent


class Action:
    def __init__(self, parent: Ros2BridgeBuilder) -> None:
        self._parent = parent

    def from_ros(self, ros_action: str, ros_qos: TopicQos) -> "ActionFromRos":
        return ActionFromRos(self._parent, ros_action, _require_topic_qos(ros_qos, "from_ros"))

    def from_bus(self, bus_action: str) -> "ActionFromBus":
        return ActionFromBus(self._parent, bus_action)


class ActionFromRos:
    def __init__(self, parent: Ros2BridgeBuilder, ros_action: str, ros_qos: TopicQos) -> None:
        self._parent = parent
        self._ros_action = ros_action
        self._ros_qos = ros_qos

    def to_bus(self, bus_action: str) -> "ActionPair":
        return ActionPair(
            self._parent, self._ros_action, bus_action, self._ros_qos, Direction.Ros2ToBus
        )


class ActionFromBus:
    def __init__(self, parent: Ros2BridgeBuilder, bus_action: str) -> None:
        self._parent = parent
        self._bus_action = bus_action

    def to_ros(self, ros_action: str, ros_qos: TopicQos) -> "ActionPair":
        return ActionPair(
            self._parent,
            ros_action,
            self._bus_action,
            _require_topic_qos(ros_qos, "to_ros"),
            Direction.BusToRos2,
        )


class ActionPair:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_action: str,
        bus_action: str,
        ros_qos: TopicQos,
        direction: Direction,
    ) -> None:
        self._parent = parent
        self._ros_action = ros_action
        self._bus_action = bus_action
        self._ros_qos = ros_qos
        self._direction = direction

    def mapper(self, mapper: Any) -> "ActionReady":
        return ActionReady(
            self._parent,
            self._ros_action,
            self._bus_action,
            mapper,
            self._direction,
            self._ros_qos,
        )


class ActionReady:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_action: str,
        bus_action: str,
        mapper: Any,
        direction: Direction,
        ros_qos: TopicQos,
    ) -> None:
        self._parent = parent
        self._ros_action = ros_action
        self._bus_action = bus_action
        self._mapper = mapper
        self._direction = direction
        self._ros_qos = ros_qos
        self._timeout = ACTION_CALL_TIMEOUT

    def timeout(self, timeout_secs: float) -> "ActionReady":
        self._timeout = timeout_secs
        return self

    def add(self) -> Ros2BridgeBuilder:
        self._parent._actions.append(
            {
                "ros_action": self._ros_action,
                "bus_action": self._bus_action,
                "mapper": self._mapper,
                "direction": self._direction,
                "timeout": self._timeout,
                "ros_qos": self._ros_qos,
            }
        )
        return self._parent


class Ros2Bridge:
    def __init__(self) -> None:
        self._bus: Any = None
        self._ros_node: Any = None
        self._executor: Any = None
        self._spin_thread: Optional[threading.Thread] = None
        self._halt = threading.Event()
        self._keep_alive: list[Any] = []
        self._lazy_routes: dict[str, dict[str, Any]] = {}
        self._eager_bus_topics: set[str] = set()
        self._subscriber_counts: dict[str, int] = {}
        self._console_live: Optional[bool] = None
        self._first_spin_at: Optional[float] = None
        self._callback_group: Any = None

    @staticmethod
    def new(name: str) -> Ros2BridgeBuilder:
        return Ros2BridgeBuilder(name)

    @classmethod
    def _from_builder(cls, builder: Ros2BridgeBuilder) -> "Ros2Bridge":
        try:
            import rclpy
            from rclpy.callback_groups import ReentrantCallbackGroup
            from rclpy.executors import MultiThreadedExecutor
        except ImportError as err:
            raise RuntimeError(
                "ROS 2 not available: source Humble/Jazzy and install rclpy"
            ) from err

        if not rclpy.ok():
            rclpy.init()

        self = cls()
        self._bus = builder._bus_factory(f"{builder._name}_bus")
        self._ros_node = rclpy.create_node(builder._name)
        self._callback_group = ReentrantCallbackGroup()
        self._executor = MultiThreadedExecutor()
        self._executor.add_node(self._ros_node)

        for route in builder._routes:
            self._wire_topic(route)
        for svc in builder._services:
            self._wire_service(svc)
        for act in builder._actions:
            self._wire_action(act)

        if self._lazy_routes:
            self._subscribe_demand()

        self._spin_thread = threading.Thread(
            target=self._ros_spin, name="ros2_bridge_spin", daemon=True
        )
        self._spin_thread.start()
        return self

    def _ros_spin(self) -> None:
        try:
            self._executor.spin()
        except Exception:  # noqa: BLE001
            pass

    def spin(self) -> None:
        while True:
            self.spin_once(None)

    def spin_once(self, timeout: Optional[float] = 0.01) -> None:
        if self._first_spin_at is None:
            self._first_spin_at = time.monotonic()
        try:
            self._bus.spin_once(timeout)
        except Exception as err:  # noqa: BLE001
            if "nothing registered" not in str(err):
                raise
        self._apply_lazy()

    def has_ros_subscription(self, bus_topic: str) -> bool:
        if bus_topic in self._lazy_routes:
            return self._lazy_routes[bus_topic]["sub"] is not None
        return bus_topic in self._eager_bus_topics

    def close(self) -> None:
        self._halt.set()
        if self._executor is not None:
            try:
                self._executor.shutdown()
            except Exception:  # noqa: BLE001
                pass
        if self._spin_thread is not None:
            self._spin_thread.join(timeout=2.0)
            self._spin_thread = None
        if self._executor is not None and self._ros_node is not None:
            try:
                self._executor.remove_node(self._ros_node)
            except Exception:  # noqa: BLE001
                pass
        if self._ros_node is not None:
            try:
                self._ros_node.destroy_node()
            except Exception:  # noqa: BLE001
                pass
            self._ros_node = None

    def __del__(self) -> None:  # pragma: no cover
        try:
            self.close()
        except Exception:
            pass

    def _subscribe_demand(self) -> None:
        def on_demand(_topic: str, payload: bytes) -> None:
            try:
                from robot_bus.robot_bus_interfaces.msg.v1 import TopicDemand
            except ImportError:
                return
            msg = TopicDemand()
            msg.ParseFromString(payload)
            self._console_live = True
            self._subscriber_counts[msg.topic] = int(msg.subscribers)

        def on_topics(_topic: str, payload: bytes) -> None:
            try:
                from robot_bus.robot_bus_interfaces.msg.v1 import TopicStatsList
            except ImportError:
                return
            msg = TopicStatsList()
            msg.ParseFromString(payload)
            self._console_live = True
            for row in msg.topics:
                self._subscriber_counts[row.name] = int(row.subscribers)

        self._keep_alive.append(self._bus.create_subscription(TOPIC_DEMAND, on_demand))
        self._keep_alive.append(self._bus.create_subscription(TOPICS_SNAPSHOT, on_topics))

    def _apply_lazy(self) -> None:
        if not self._lazy_routes:
            return
        if self._console_live is None and self._first_spin_at is not None:
            if time.monotonic() - self._first_spin_at >= CONSOLE_DETECT_TIMEOUT:
                self._console_live = False
        for bus_topic, route in self._lazy_routes.items():
            n = self._subscriber_counts.get(bus_topic, 0)
            want = should_enable_ros_subscription(True, self._console_live, n)
            sub = route["sub"]
            if want and sub is None:
                route["sub"] = route["create"]()
            elif not want and sub is not None:
                try:
                    self._ros_node.destroy_subscription(sub)
                except Exception:  # noqa: BLE001
                    pass
                route["sub"] = None

    def _wire_topic(self, route: dict[str, Any]) -> None:
        mapper = route["mapper"]
        ros_topic = route["ros_topic"]
        bus_topic = route["bus_topic"]
        direction = route["direction"]
        lazy = route["lazy"]

        if callable(getattr(mapper, "attach", None)) and not _topic_supports_lazy(mapper):
            ctx = TopicWireContext(
                self._ros_node,
                self._bus,
                ros_topic,
                bus_topic,
                direction,
                self._keep_alive,
                qos=_ros_qos(route["ros_qos"]),
                bus_qos_depth=route["bus_qos"].depth,
            )
            mapper.attach(ctx)
            if direction == Direction.Ros2ToBus:
                self._eager_bus_topics.add(bus_topic)
            return

        msg_type = mapper.ros_msg_type()
        ros_qos = _ros_qos(route["ros_qos"])
        bus_depth = route["bus_qos"].depth
        if direction == Direction.BusToRos2:
            ros_pub = self._ros_node.create_publisher(msg_type, ros_topic, ros_qos)

            def on_bus(_topic: str, payload: bytes, m=mapper, pub=ros_pub) -> None:
                try:
                    pub.publish(m.bus_to_ros(payload))
                except Exception:
                    pass

            sub_kw: dict[str, Any] = {"qos_depth": bus_depth}
            self._keep_alive.append(self._bus.create_subscription(bus_topic, on_bus, **sub_kw))
            self._keep_alive.append(ros_pub)
            return

        pub_kw: dict[str, Any] = {"qos_depth": bus_depth}
        bus_pub = self._bus.create_publisher(bus_topic, **pub_kw)
        lock = threading.Lock()

        def create_sub(
            m=mapper, t=msg_type, rt=ros_topic, pub=bus_pub, mtx=lock, rq=ros_qos
        ) -> Any:
            def on_ros(msg, pub=pub, mtx=mtx, m=m) -> None:
                try:
                    payload = m.ros_to_bus(msg)
                    with mtx:
                        pub.publish(payload)
                except Exception:
                    pass

            return self._ros_node.create_subscription(t, rt, on_ros, rq)

        if lazy:
            self._lazy_routes[bus_topic] = {"create": create_sub, "sub": None}
            self._keep_alive.append(bus_pub)
            return

        self._keep_alive.append(create_sub())
        self._keep_alive.append(bus_pub)
        self._eager_bus_topics.add(bus_topic)

    def _wire_service(self, route: dict[str, Any]) -> None:
        mapper = route["mapper"]
        if callable(getattr(mapper, "attach", None)) and not callable(
            getattr(mapper, "ros_srv_type", None)
        ):
            ctx = ServiceWireContext(
                self._ros_node,
                self._bus,
                route["ros_service"],
                route["bus_service"],
                route["direction"],
                route["timeout"],
                self._callback_group,
                self._keep_alive,
                _ros_service_qos(route["ros_qos"]),
            )
            mapper.attach(ctx)
            return

        srv_type = mapper.ros_srv_type()
        timeout = route["timeout"]
        ros_qos = _ros_service_qos(route["ros_qos"])
        if route["direction"] == Direction.Ros2ToBus:
            bus_client = self._bus.create_client(route["bus_service"])
            lock = threading.Lock()

            def on_ros(request, response, m=mapper, client=bus_client, mtx=lock) -> Any:
                try:
                    req_bytes = m.ros_req_to_bus(request)
                    with mtx:
                        resp_bytes = client.call(req_bytes, timeout)
                    out = m.bus_resp_to_ros(resp_bytes)
                    _copy_msg(out, response)
                except Exception as err:  # noqa: BLE001
                    err_fn = getattr(m, "error_response", None)
                    if callable(err_fn):
                        _copy_msg(err_fn(f"bus call failed: {err}"), response)
                return response

            srv = self._ros_node.create_service(
                srv_type,
                route["ros_service"],
                on_ros,
                qos_profile=ros_qos,
                callback_group=self._callback_group,
            )
            self._keep_alive.extend((bus_client, srv, lock))
            return

        ros_client = self._ros_node.create_client(
            srv_type,
            route["ros_service"],
            qos_profile=ros_qos,
            callback_group=self._callback_group,
        )

        def on_bus(payload: bytes, m=mapper, client=ros_client) -> bytes:
            if not client.wait_for_service(timeout_sec=timeout):
                err_fn = getattr(m, "error_response", None)
                if callable(err_fn):
                    return m.ros_resp_to_bus(err_fn("timed out waiting for ROS service"))
                return b""
            req = m.bus_req_to_ros(payload)
            future = client.call_async(req)
            try:
                resp = future.result(timeout=timeout)
            except Exception:
                err_fn = getattr(m, "error_response", None)
                if callable(err_fn):
                    return m.ros_resp_to_bus(err_fn("timed out waiting for ROS response"))
                return b""
            return m.ros_resp_to_bus(resp)

        self._keep_alive.append(self._bus.create_service(route["bus_service"], on_bus))
        self._keep_alive.append(ros_client)

    def _wire_action(self, route: dict[str, Any]) -> None:
        mapper = route["mapper"]
        if callable(getattr(mapper, "attach", None)) and not callable(
            getattr(mapper, "ros_action_type", None)
        ):
            ctx = ActionWireContext(
                self._ros_node,
                self._bus,
                route["ros_action"],
                route["bus_action"],
                route["direction"],
                route["timeout"],
                self._callback_group,
                self._keep_alive,
                route["ros_qos"],
            )
            mapper.attach(ctx)
            return

        from rclpy.action import ActionClient, ActionServer, CancelResponse, GoalResponse

        act_type = mapper.ros_action_type()
        timeout = route["timeout"]
        srv_qos = _ros_service_qos(route["ros_qos"])
        fb_qos = _ros_qos(route["ros_qos"])
        if route["direction"] == Direction.Ros2ToBus:
            bus_client = self._bus.create_action_client(route["bus_action"])
            lock = threading.Lock()

            def execute_cb(goal_handle, m=mapper, client=bus_client, mtx=lock):
                goal = goal_handle.request
                try:
                    goal_bytes = m.ros_goal_to_bus(goal)

                    def on_fb(body: bytes, gh=goal_handle, mm=m) -> None:
                        try:
                            fb = mm.bus_feedback_to_ros(body)
                            gh.publish_feedback(fb)
                        except Exception:
                            pass

                    with mtx:
                        handle = client.send_goal(
                            goal_bytes, timeout=timeout, feedback_callback=on_fb
                        )
                    result_bytes = handle.result(timeout=timeout)
                    result = m.bus_result_to_ros(result_bytes)
                    goal_handle.succeed()
                    return result
                except Exception:
                    goal_handle.abort()
                    return act_type.Result()

            server = ActionServer(
                self._ros_node,
                act_type,
                route["ros_action"],
                execute_callback=execute_cb,
                goal_callback=lambda _g: GoalResponse.ACCEPT,
                cancel_callback=lambda _g: CancelResponse.ACCEPT,
                callback_group=self._callback_group,
                goal_service_qos_profile=srv_qos,
                result_service_qos_profile=srv_qos,
                cancel_service_qos_profile=srv_qos,
                feedback_pub_qos_profile=fb_qos,
            )
            self._keep_alive.extend((bus_client, server, lock))
            return

        ros_client = ActionClient(
            self._ros_node,
            act_type,
            route["ros_action"],
            callback_group=self._callback_group,
            goal_service_qos_profile=srv_qos,
            result_service_qos_profile=srv_qos,
            cancel_service_qos_profile=srv_qos,
            feedback_sub_qos_profile=fb_qos,
        )

        def on_bus(payload: bytes, m=mapper, client=ros_client) -> list:
            goal = m.bus_goal_to_ros(payload)
            if not client.wait_for_server(timeout_sec=timeout):
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            feedbacks: list[bytes] = []
            fb_lock = threading.Lock()

            def on_fb(fb_msg, mm=m) -> None:
                try:
                    raw = mm.ros_feedback_to_bus(fb_msg.feedback)
                    with fb_lock:
                        feedbacks.append(raw)
                except Exception:
                    pass

            send_future = client.send_goal_async(goal, feedback_callback=on_fb)
            try:
                goal_handle = send_future.result(timeout=timeout)
            except Exception:
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            if goal_handle is None:
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            result_future = goal_handle.get_result_async()
            try:
                wrapped = result_future.result(timeout=timeout)
            except Exception:
                return [("RESULT", m.ros_result_to_bus(act_type.Result()))]
            phases: list[tuple[str, bytes]] = []
            with fb_lock:
                for fb in feedbacks:
                    phases.append(("FEEDBACK", fb))
            result = wrapped.result if wrapped is not None else act_type.Result()
            phases.append(("RESULT", m.ros_result_to_bus(result)))
            return phases

        self._keep_alive.append(self._bus.create_action_server(route["bus_action"], on_bus))
        self._keep_alive.append(ros_client)


def _copy_msg(src: Any, dest: Any) -> None:
    for slot in getattr(dest, "__slots__", ()):
        name = slot.lstrip("_")
        if hasattr(src, name):
            try:
                setattr(dest, name, getattr(src, name))
            except Exception:
                pass
    if hasattr(src, "get_fields_and_field_types"):
        for name in src.get_fields_and_field_types():
            if hasattr(dest, name):
                try:
                    setattr(dest, name, getattr(src, name))
                except Exception:
                    pass
