"""Service / action fluent stages for Ros2BridgeBuilder."""

from __future__ import annotations

from typing import Any

from .config import (
    ACTION_CALL_TIMEOUT,
    SERVICE_CALL_TIMEOUT,
    Direction,
    Ros2BridgeBuilder,
    TopicQos,
    _require_bus_best_effort,
    _require_topic_qos,
)

class Service:
    def __init__(self, parent: Ros2BridgeBuilder) -> None:
        self._parent = parent

    def from_ros(self, ros_service: str, ros_qos: TopicQos) -> "ServiceFromRos":
        return ServiceFromRos(self._parent, ros_service, _require_topic_qos(ros_qos, "from_ros"))

    def from_bus(self, bus_service: str, bus_qos: TopicQos) -> "ServiceFromBus":
        qos = _require_topic_qos(bus_qos, "from_bus")
        _require_bus_best_effort(qos)
        return ServiceFromBus(self._parent, bus_service, qos)


class ServiceFromRos:
    def __init__(self, parent: Ros2BridgeBuilder, ros_service: str, ros_qos: TopicQos) -> None:
        self._parent = parent
        self._ros_service = ros_service
        self._ros_qos = ros_qos

    def to_bus(self, bus_service: str, bus_qos: TopicQos) -> "ServicePair":
        qos = _require_topic_qos(bus_qos, "to_bus")
        _require_bus_best_effort(qos)
        return ServicePair(
            self._parent,
            self._ros_service,
            bus_service,
            self._ros_qos,
            qos,
            Direction.Ros2ToBus,
        )


class ServiceFromBus:
    def __init__(self, parent: Ros2BridgeBuilder, bus_service: str, bus_qos: TopicQos) -> None:
        self._parent = parent
        self._bus_service = bus_service
        self._bus_qos = bus_qos

    def to_ros(self, ros_service: str, ros_qos: TopicQos) -> "ServicePair":
        return ServicePair(
            self._parent,
            ros_service,
            self._bus_service,
            _require_topic_qos(ros_qos, "to_ros"),
            self._bus_qos,
            Direction.BusToRos2,
        )


class ServicePair:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_service: str,
        bus_service: str,
        ros_qos: TopicQos,
        bus_qos: TopicQos,
        direction: Direction,
    ) -> None:
        self._parent = parent
        self._ros_service = ros_service
        self._bus_service = bus_service
        self._ros_qos = ros_qos
        self._bus_qos = bus_qos
        self._direction = direction

    def mapper(self, mapper: Any) -> "ServiceReady":
        return ServiceReady(
            self._parent,
            self._ros_service,
            self._bus_service,
            mapper,
            self._direction,
            self._ros_qos,
            self._bus_qos,
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
        bus_qos: TopicQos,
    ) -> None:
        self._parent = parent
        self._ros_service = ros_service
        self._bus_service = bus_service
        self._mapper = mapper
        self._direction = direction
        self._ros_qos = ros_qos
        self._bus_qos = bus_qos
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
                "bus_qos": self._bus_qos,
            }
        )
        return self._parent


class Action:
    def __init__(self, parent: Ros2BridgeBuilder) -> None:
        self._parent = parent

    def from_ros(self, ros_action: str, ros_qos: TopicQos) -> "ActionFromRos":
        return ActionFromRos(self._parent, ros_action, _require_topic_qos(ros_qos, "from_ros"))

    def from_bus(self, bus_action: str, bus_qos: TopicQos) -> "ActionFromBus":
        qos = _require_topic_qos(bus_qos, "from_bus")
        _require_bus_best_effort(qos)
        return ActionFromBus(self._parent, bus_action, qos)


class ActionFromRos:
    def __init__(self, parent: Ros2BridgeBuilder, ros_action: str, ros_qos: TopicQos) -> None:
        self._parent = parent
        self._ros_action = ros_action
        self._ros_qos = ros_qos

    def to_bus(self, bus_action: str, bus_qos: TopicQos) -> "ActionPair":
        qos = _require_topic_qos(bus_qos, "to_bus")
        _require_bus_best_effort(qos)
        return ActionPair(
            self._parent,
            self._ros_action,
            bus_action,
            self._ros_qos,
            qos,
            Direction.Ros2ToBus,
        )


class ActionFromBus:
    def __init__(self, parent: Ros2BridgeBuilder, bus_action: str, bus_qos: TopicQos) -> None:
        self._parent = parent
        self._bus_action = bus_action
        self._bus_qos = bus_qos

    def to_ros(self, ros_action: str, ros_qos: TopicQos) -> "ActionPair":
        return ActionPair(
            self._parent,
            ros_action,
            self._bus_action,
            _require_topic_qos(ros_qos, "to_ros"),
            self._bus_qos,
            Direction.BusToRos2,
        )


class ActionPair:
    def __init__(
        self,
        parent: Ros2BridgeBuilder,
        ros_action: str,
        bus_action: str,
        ros_qos: TopicQos,
        bus_qos: TopicQos,
        direction: Direction,
    ) -> None:
        self._parent = parent
        self._ros_action = ros_action
        self._bus_action = bus_action
        self._ros_qos = ros_qos
        self._bus_qos = bus_qos
        self._direction = direction

    def mapper(self, mapper: Any) -> "ActionReady":
        return ActionReady(
            self._parent,
            self._ros_action,
            self._bus_action,
            mapper,
            self._direction,
            self._ros_qos,
            self._bus_qos,
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
        bus_qos: TopicQos,
    ) -> None:
        self._parent = parent
        self._ros_action = ros_action
        self._bus_action = bus_action
        self._mapper = mapper
        self._direction = direction
        self._ros_qos = ros_qos
        self._bus_qos = bus_qos
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
                "bus_qos": self._bus_qos,
            }
        )
        return self._parent

