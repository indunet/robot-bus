"""Generated mapper for `std_msgs/msg/ByteMultiArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.multi_array_layout import multi_array_layout_to_bus, multi_array_layout_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.std_msgs.msg.v1 import ByteMultiArray as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def byte_multi_array_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.layout.CopyFrom(multi_array_layout_to_bus(msg.layout))
    bus.data = bytes(msg.data)
    return bus


def byte_multi_array_to_ros(bus):
    from std_msgs.msg import ByteMultiArray as RosMsg

    out = RosMsg()
    out.layout = multi_array_layout_to_ros(bus.layout)
    out.data = bytes(bus.data)
    return out


class StdMsgsByteMultiArrayMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from std_msgs.msg import ByteMultiArray as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return byte_multi_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return byte_multi_array_to_ros(bus)
