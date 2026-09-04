"""Generated mapper for `nav2_msgs/msg/RouteEdge`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import RouteEdge as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def route_edge_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import RouteEdge as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return route_edge_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return route_edge_to_ros(bus)
