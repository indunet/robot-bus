"""Generated mapper for `foxglove_msgs/msg/CompressedImage`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import CompressedImage as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def compressed_image_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.data = bytes(msg.data)
    bus.format = str(msg.format)
    return bus


def compressed_image_to_ros(bus):
    from foxglove_msgs.msg import CompressedImage as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.data = bytes(bus.data)
    out.format = str(bus.format)
    return out


class FoxgloveMsgsCompressedImageMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import CompressedImage as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return compressed_image_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return compressed_image_to_ros(bus)
