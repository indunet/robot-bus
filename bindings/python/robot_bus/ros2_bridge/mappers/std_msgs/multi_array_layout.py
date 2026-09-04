"""Generated mapper for `std_msgs/msg/MultiArrayLayout`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.multi_array_dimension import multi_array_dimension_to_bus, multi_array_dimension_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.std_msgs.msg.v1 import MultiArrayLayout as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def multi_array_layout_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.dim.extend([multi_array_dimension_to_bus(x) for x in msg.dim])
    bus.data_offset = msg.data_offset
    return bus


def multi_array_layout_to_ros(bus):
    from std_msgs.msg import MultiArrayLayout as RosMsg

    out = RosMsg()
    out.dim = [multi_array_dimension_to_ros(x) for x in bus.dim]
    out.data_offset = bus.data_offset
    return out


class StdMsgsMultiArrayLayoutMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from std_msgs.msg import MultiArrayLayout as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return multi_array_layout_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return multi_array_layout_to_ros(bus)
