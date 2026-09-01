"""Generated mapper for `foxglove_msgs/msg/Point3InFrame`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point3 import point3_to_bus, point3_to_ros

def point3_in_frame_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import Point3InFrame as BusMsg

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
    def ros_msg_type(self):
        from foxglove_msgs.msg import Point3InFrame as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point3_in_frame_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import Point3InFrame as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point3_in_frame_to_ros(bus)
