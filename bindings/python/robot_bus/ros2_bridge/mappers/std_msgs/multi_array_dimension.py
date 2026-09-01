"""Generated mapper for `std_msgs/msg/MultiArrayDimension`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def multi_array_dimension_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import MultiArrayDimension as BusMsg

    bus = BusMsg()
    bus.label = str(msg.label)
    bus.size = msg.size
    bus.stride = msg.stride
    return bus


def multi_array_dimension_to_ros(bus):
    from std_msgs.msg import MultiArrayDimension as RosMsg

    out = RosMsg()
    out.label = str(bus.label)
    out.size = bus.size
    out.stride = bus.stride
    return out


class StdMsgsMultiArrayDimensionMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/MultiArrayDimension"

    def ros_msg_type(self):
        from std_msgs.msg import MultiArrayDimension as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return multi_array_dimension_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import MultiArrayDimension as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return multi_array_dimension_to_ros(bus)
