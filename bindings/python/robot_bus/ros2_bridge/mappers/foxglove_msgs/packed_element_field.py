"""Generated mapper for `foxglove_msgs/msg/PackedElementField`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import PackedElementField as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def packed_element_field_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.name = str(msg.name)
    bus.offset = msg.offset
    bus.type = int(msg.type)
    return bus


def packed_element_field_to_ros(bus):
    from foxglove_msgs.msg import PackedElementField as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.offset = bus.offset
    out.type = int(bus.type)
    return out


class FoxgloveMsgsPackedElementFieldMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import PackedElementField as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return packed_element_field_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return packed_element_field_to_ros(bus)
