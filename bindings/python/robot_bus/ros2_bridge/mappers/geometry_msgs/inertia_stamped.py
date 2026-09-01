"""Generated mapper for `geometry_msgs/msg/InertiaStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.inertia import inertia_to_bus, inertia_to_ros

def inertia_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import InertiaStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.inertia.CopyFrom(inertia_to_bus(msg.inertia))
    return bus


def inertia_stamped_to_ros(bus):
    from geometry_msgs.msg import InertiaStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.inertia = inertia_to_ros(bus.inertia)
    return out


class GeometryMsgsInertiaStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/InertiaStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import InertiaStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return inertia_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import InertiaStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return inertia_stamped_to_ros(bus)
