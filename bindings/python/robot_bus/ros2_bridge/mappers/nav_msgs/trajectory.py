"""Generated mapper for `nav_msgs/msg/Trajectory`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.nav_msgs.trajectory_point import trajectory_point_to_bus, trajectory_point_to_ros

def trajectory_to_bus(msg):
    from robot_bus.nav_msgs.msg.v1 import Trajectory as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.points.extend([trajectory_point_to_bus(x) for x in msg.points])
    return bus


def trajectory_to_ros(bus):
    from nav_msgs.msg import Trajectory as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.points = [trajectory_point_to_ros(x) for x in bus.points]
    return out


class NavMsgsTrajectoryMapper:
    def ros_msg_type(self):
        from nav_msgs.msg import Trajectory as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return trajectory_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav_msgs.msg.v1 import Trajectory as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return trajectory_to_ros(bus)
