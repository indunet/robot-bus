"""Generated mapper for `nav_msgs/msg/Goals`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_stamped import pose_stamped_to_bus, pose_stamped_to_ros

def goals_to_bus(msg):
    from robot_bus.nav_msgs.msg.v1 import Goals as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.goals.extend([pose_stamped_to_bus(x) for x in msg.goals])
    return bus


def goals_to_ros(bus):
    from nav_msgs.msg import Goals as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.goals = [pose_stamped_to_ros(x) for x in bus.goals]
    return out


class NavMsgsGoalsMapper:
    def type_name(self) -> str:
        return "nav_msgs/msg/Goals"

    def ros_msg_type(self):
        from nav_msgs.msg import Goals as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return goals_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav_msgs.msg.v1 import Goals as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return goals_to_ros(bus)
