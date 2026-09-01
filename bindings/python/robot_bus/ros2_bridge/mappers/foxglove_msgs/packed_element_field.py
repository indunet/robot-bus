"""Generated mapper for `foxglove_msgs/msg/PackedElementField`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def packed_element_field_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import PackedElementField as BusMsg

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
    def type_name(self) -> str:
        return "foxglove_msgs/msg/PackedElementField"

    def ros_msg_type(self):
        from foxglove_msgs.msg import PackedElementField as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return packed_element_field_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import PackedElementField as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return packed_element_field_to_ros(bus)
