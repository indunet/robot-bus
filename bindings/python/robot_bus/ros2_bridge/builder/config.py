"""Fluent `Ros2Bridge` builder (rclpy + robot_bus.Node)."""

from __future__ import annotations

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
    """KeepLast depth plus reliability and optional ROS durability.

    Same type on ROS and bus endpoints for topics, services, and actions.
    ROS honors depth + reliability + durability. Bus uses depth as ZMQ HWM
    and must be ``.best_effort()`` (no DDS reliability); durability is ignored
    on bus.
    """

    def __init__(
        self, depth: int, best_effort: bool, transient_local: bool = False
    ) -> None:
        self._depth = int(depth)
        self._best_effort = bool(best_effort)
        self._transient_local = bool(transient_local)

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

    @property
    def is_transient_local(self) -> bool:
        return self._transient_local

    @property
    def is_volatile(self) -> bool:
        return not self._transient_local

    def transient_local(self) -> "TopicQos":
        """ROS ``TRANSIENT_LOCAL`` (latch), e.g. ``/tf_static``."""
        return TopicQos(self._depth, self._best_effort, True)

    def volatile(self) -> "TopicQos":
        """ROS ``VOLATILE`` (default). New subscribers only see later samples."""
        return TopicQos(self._depth, self._best_effort, False)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, TopicQos):
            return NotImplemented
        return (
            self._depth == other._depth
            and self._best_effort == other._best_effort
            and self._transient_local == other._transient_local
        )


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
        DurabilityPolicy,
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
    profile.durability = (
        DurabilityPolicy.TRANSIENT_LOCAL
        if qos.is_transient_local
        else DurabilityPolicy.VOLATILE
    )
    return profile


def _ros_service_qos(qos: TopicQos) -> Any:
    from rclpy.qos import (
        DurabilityPolicy,
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
        durability=(
            DurabilityPolicy.TRANSIENT_LOCAL
            if qos.is_transient_local
            else DurabilityPolicy.VOLATILE
        ),
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
            "(bus has no DDS reliability)"
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
        bus_qos_depth: Optional[int] = None,
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
        self.bus_qos_depth = bus_qos_depth

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
        bus_qos_depth: Optional[int] = None,
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
        self.bus_qos_depth = bus_qos_depth

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
        from .topic import FromRos
        return FromRos(self, ros_topic, _require_topic_qos(ros_qos, "from_ros"))

    def from_bus(self, bus_topic: str, bus_qos: TopicQos) -> "FromBus":
        qos = _require_topic_qos(bus_qos, "from_bus")
        _require_bus_best_effort(qos)
        from .topic import FromBus
        return FromBus(self, bus_topic, qos)

    def service(self) -> "Service":
        from .rpc import Service
        return Service(self)

    def action(self) -> "Action":
        from .rpc import Action
        return Action(self)

    def build(self) -> "Ros2Bridge":
        if not self._routes and not self._services and not self._actions:
            raise ValueError(
                "Ros2Bridge requires at least one topic route, service, or action"
            )
        from .bridge import Ros2Bridge
        return Ros2Bridge._from_builder(self)
