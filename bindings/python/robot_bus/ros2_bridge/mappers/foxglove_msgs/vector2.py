"""Generated mapper for `foxglove_msgs/msg/Vector2`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def vector2_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import Vector2 as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    return bus


def vector2_to_ros(bus):
    from foxglove_msgs.msg import Vector2 as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    return out


class FoxgloveMsgsVector2Mapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import Vector2 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return vector2_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import Vector2 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return vector2_to_ros(bus)
