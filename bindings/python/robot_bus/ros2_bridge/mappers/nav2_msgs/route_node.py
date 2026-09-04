"""Generated mapper for `nav2_msgs/msg/RouteNode`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import RouteNode as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def route_node_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.nodeid = msg.nodeid
    bus.position.CopyFrom(point_to_bus(msg.position))
    return bus


def route_node_to_ros(bus):
    from nav2_msgs.msg import RouteNode as RosMsg

    out = RosMsg()
    out.nodeid = bus.nodeid
    out.position = point_to_ros(bus.position)
    return out


class Nav2MsgsRouteNodeMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import RouteNode as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return route_node_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return route_node_to_ros(bus)
