"""Generated mapper for `foxglove_msgs/msg/RawImage`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import RawImage as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def raw_image_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import RawImage as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return raw_image_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return raw_image_to_ros(bus)
