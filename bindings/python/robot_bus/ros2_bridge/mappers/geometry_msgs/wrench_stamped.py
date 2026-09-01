"""Generated mapper for `geometry_msgs/msg/WrenchStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.wrench import wrench_to_bus, wrench_to_ros

def wrench_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import WrenchStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.wrench.CopyFrom(wrench_to_bus(msg.wrench))
    return bus


def wrench_stamped_to_ros(bus):
    from geometry_msgs.msg import WrenchStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.wrench = wrench_to_ros(bus.wrench)
    return out


class GeometryMsgsWrenchStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/WrenchStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import WrenchStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return wrench_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import WrenchStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return wrench_stamped_to_ros(bus)
