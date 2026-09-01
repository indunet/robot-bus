"""Generated mapper for `geometry_msgs/msg/TwistStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros

def twist_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import TwistStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.twist.CopyFrom(twist_to_bus(msg.twist))
    return bus


def twist_stamped_to_ros(bus):
    from geometry_msgs.msg import TwistStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.twist = twist_to_ros(bus.twist)
    return out


class GeometryMsgsTwistStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/TwistStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import TwistStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return twist_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import TwistStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return twist_stamped_to_ros(bus)
