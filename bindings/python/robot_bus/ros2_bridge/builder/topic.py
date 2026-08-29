"""Topic fluent stages for Ros2BridgeBuilder."""

from __future__ import annotations

from typing import Any

from .config import (
    Direction,
    Ros2BridgeBuilder,
    TopicQos,
    _require_bus_best_effort,
    _require_topic_qos,
    _topic_supports_lazy,
)

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

