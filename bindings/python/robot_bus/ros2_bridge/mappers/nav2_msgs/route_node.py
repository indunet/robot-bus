"""Generated mapper for `nav2_msgs/msg/RouteNode`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

def route_node_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import RouteNode as BusMsg

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
    def type_name(self) -> str:
        return "nav2_msgs/msg/RouteNode"

    def ros_msg_type(self):
        from nav2_msgs.msg import RouteNode as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return route_node_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import RouteNode as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return route_node_to_ros(bus)
