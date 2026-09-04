"""Generated mapper for `visualization_msgs/msg/UVCoordinate`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.visualization_msgs.msg.v1 import UVCoordinate as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def uv_coordinate_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.u = msg.u
    bus.v = msg.v
    return bus


def uv_coordinate_to_ros(bus):
    from visualization_msgs.msg import UVCoordinate as RosMsg

    out = RosMsg()
    out.u = bus.u
    out.v = bus.v
    return out


class VisualizationMsgsUvCoordinateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from visualization_msgs.msg import UVCoordinate as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return uv_coordinate_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return uv_coordinate_to_ros(bus)
