"""Generated mapper for `geometry_msgs/msg/Wrench`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

def wrench_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Wrench as BusMsg

    bus = BusMsg()
    bus.force.CopyFrom(vector3_to_bus(msg.force))
    bus.torque.CopyFrom(vector3_to_bus(msg.torque))
    return bus


def wrench_to_ros(bus):
    from geometry_msgs.msg import Wrench as RosMsg

    out = RosMsg()
    out.force = vector3_to_ros(bus.force)
    out.torque = vector3_to_ros(bus.torque)
    return out


class GeometryMsgsWrenchMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import Wrench as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return wrench_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Wrench as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return wrench_to_ros(bus)
