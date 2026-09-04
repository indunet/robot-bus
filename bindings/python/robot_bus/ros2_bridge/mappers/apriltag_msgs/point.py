"""Generated mapper for `apriltag_msgs/msg/Point`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.apriltag_msgs.msg.v1 import Point as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def point_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    return bus


def point_to_ros(bus):
    from apriltag_msgs.msg import Point as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    return out


class ApriltagMsgsPointMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from apriltag_msgs.msg import Point as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return point_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_to_ros(bus)
