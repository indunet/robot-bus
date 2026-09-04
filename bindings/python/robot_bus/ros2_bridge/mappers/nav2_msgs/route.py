"""Generated mapper for `nav2_msgs/msg/Route`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.nav2_msgs.route_node import route_node_to_bus, route_node_to_ros
from robot_bus.ros2_bridge.mappers.nav2_msgs.route_edge import route_edge_to_bus, route_edge_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import Route as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def route_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.nodes.extend([route_node_to_bus(x) for x in msg.nodes])
    bus.edges.extend([route_edge_to_bus(x) for x in msg.edges])
    return bus


def route_to_ros(bus):
    from nav2_msgs.msg import Route as RosMsg

    out = RosMsg()
    out.nodes = [route_node_to_ros(x) for x in bus.nodes]
    out.edges = [route_edge_to_ros(x) for x in bus.edges]
    return out


class Nav2MsgsRouteMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import Route as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return route_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return route_to_ros(bus)
