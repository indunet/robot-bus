"""Generated mapper for `geometry_msgs/msg/Pose2D`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.geometry_msgs.msg.v1 import Pose2D as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def pose2_d_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    bus.theta = msg.theta
    return bus


def pose2_d_to_ros(bus):
    from geometry_msgs.msg import Pose2D as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    out.theta = bus.theta
    return out


class GeometryMsgsPose2DMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from geometry_msgs.msg import Pose2D as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return pose2_d_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return pose2_d_to_ros(bus)
