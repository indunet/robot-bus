"""Generated mapper for `geometry_msgs/msg/Twist`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

def twist_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Twist as BusMsg

    bus = BusMsg()
    bus.linear.CopyFrom(vector3_to_bus(msg.linear))
    bus.angular.CopyFrom(vector3_to_bus(msg.angular))
    return bus


def twist_to_ros(bus):
    from geometry_msgs.msg import Twist as RosMsg

    out = RosMsg()
    out.linear = vector3_to_ros(bus.linear)
    out.angular = vector3_to_ros(bus.angular)
    return out


class GeometryMsgsTwistMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import Twist as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return twist_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Twist as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return twist_to_ros(bus)
