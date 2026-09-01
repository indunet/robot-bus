"""Generated mapper for `foxglove_msgs/msg/LinePrimitive`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point3 import point3_to_bus, point3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros

def line_primitive_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import LinePrimitive as BusMsg

    bus = BusMsg()
    bus.type = int(msg.type)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.thickness = msg.thickness
    bus.scale_invariant = msg.scale_invariant
    bus.points.extend([point3_to_bus(x) for x in msg.points])
    bus.color.CopyFrom(color_to_bus(msg.color))
    bus.colors.extend([color_to_bus(x) for x in msg.colors])
    bus.indices.extend(list(msg.indices))
    return bus


def line_primitive_to_ros(bus):
    from foxglove_msgs.msg import LinePrimitive as RosMsg

    out = RosMsg()
    out.type = int(bus.type)
    out.pose = pose_to_ros(bus.pose)
    out.thickness = bus.thickness
    out.scale_invariant = bus.scale_invariant
    out.points = [point3_to_ros(x) for x in bus.points]
    out.color = color_to_ros(bus.color)
    out.colors = [color_to_ros(x) for x in bus.colors]
    out.indices = list(bus.indices)
    return out


class FoxgloveMsgsLinePrimitiveMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import LinePrimitive as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return line_primitive_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import LinePrimitive as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return line_primitive_to_ros(bus)
