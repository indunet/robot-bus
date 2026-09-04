"""Generated mapper for `foxglove_msgs/msg/Point3InFrame`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point3 import point3_to_bus, point3_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import Point3InFrame as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def point3_in_frame_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.point.CopyFrom(point3_to_bus(msg.point))
    return bus


def point3_in_frame_to_ros(bus):
    from foxglove_msgs.msg import Point3InFrame as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.point = point3_to_ros(bus.point)
    return out


class FoxgloveMsgsPoint3InFrameMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import Point3InFrame as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return point3_in_frame_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return point3_in_frame_to_ros(bus)
