"""Generated mapper for `foxglove_msgs/msg/RawImage`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def raw_image_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import RawImage as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.width = msg.width
    bus.height = msg.height
    bus.encoding = str(msg.encoding)
    bus.step = msg.step
    bus.data = bytes(msg.data)
    return bus


def raw_image_to_ros(bus):
    from foxglove_msgs.msg import RawImage as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.width = bus.width
    out.height = bus.height
    out.encoding = str(bus.encoding)
    out.step = bus.step
    out.data = bytes(bus.data)
    return out


class FoxgloveMsgsRawImageMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/RawImage"

    def ros_msg_type(self):
        from foxglove_msgs.msg import RawImage as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return raw_image_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import RawImage as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return raw_image_to_ros(bus)
