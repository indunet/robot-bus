"""Generated mapper for `foxglove_msgs/msg/CubePrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros

def cube_primitive_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import CubePrimitive as BusMsg

    bus = BusMsg()
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.size.CopyFrom(vector3_to_bus(msg.size))
    bus.color.CopyFrom(color_to_bus(msg.color))
    return bus


def cube_primitive_to_ros(bus):
    from foxglove_msgs.msg import CubePrimitive as RosMsg

    out = RosMsg()
    out.pose = pose_to_ros(bus.pose)
    out.size = vector3_to_ros(bus.size)
    out.color = color_to_ros(bus.color)
    return out


class FoxgloveMsgsCubePrimitiveMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/CubePrimitive"

    def ros_msg_type(self):
        from foxglove_msgs.msg import CubePrimitive as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return cube_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import CubePrimitive as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return cube_primitive_to_ros(bus)
