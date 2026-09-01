"""Generated mapper for `geometry_msgs/msg/Transform`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.quaternion import quaternion_to_bus, quaternion_to_ros

def transform_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Transform as BusMsg

    bus = BusMsg()
    bus.translation.CopyFrom(vector3_to_bus(msg.translation))
    bus.rotation.CopyFrom(quaternion_to_bus(msg.rotation))
    return bus


def transform_to_ros(bus):
    from geometry_msgs.msg import Transform as RosMsg

    out = RosMsg()
    out.translation = vector3_to_ros(bus.translation)
    out.rotation = quaternion_to_ros(bus.rotation)
    return out


class GeometryMsgsTransformMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import Transform as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return transform_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Transform as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return transform_to_ros(bus)
