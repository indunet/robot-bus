"""Generated mapper for `foxglove_msgs/msg/ModelPrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros

def model_primitive_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import ModelPrimitive as BusMsg

    bus = BusMsg()
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.scale.CopyFrom(vector3_to_bus(msg.scale))
    bus.color.CopyFrom(color_to_bus(msg.color))
    bus.override_color = msg.override_color
    bus.url = str(msg.url)
    bus.media_type = str(msg.media_type)
    bus.data = bytes(msg.data)
    return bus


def model_primitive_to_ros(bus):
    from foxglove_msgs.msg import ModelPrimitive as RosMsg

    out = RosMsg()
    out.pose = pose_to_ros(bus.pose)
    out.scale = vector3_to_ros(bus.scale)
    out.color = color_to_ros(bus.color)
    out.override_color = bus.override_color
    out.url = str(bus.url)
    out.media_type = str(bus.media_type)
    out.data = bytes(bus.data)
    return out


class FoxgloveMsgsModelPrimitiveMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import ModelPrimitive as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return model_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import ModelPrimitive as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return model_primitive_to_ros(bus)
