"""Generated mapper for `nav2_msgs/msg/RouteEdge`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def route_edge_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import RouteEdge as BusMsg

    bus = BusMsg()
    bus.edgeid = msg.edgeid
    bus.start = msg.start
    bus.end = msg.end
    return bus


def route_edge_to_ros(bus):
    from nav2_msgs.msg import RouteEdge as RosMsg

    out = RosMsg()
    out.edgeid = bus.edgeid
    out.start = bus.start
    out.end = bus.end
    return out


class Nav2MsgsRouteEdgeMapper:
    def ros_msg_type(self):
        from nav2_msgs.msg import RouteEdge as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return route_edge_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import RouteEdge as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return route_edge_to_ros(bus)
