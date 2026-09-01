"""Generated mapper for `nav2_msgs/msg/MissedWaypoint`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_stamped import pose_stamped_to_bus, pose_stamped_to_ros

def missed_waypoint_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import MissedWaypoint as BusMsg

    bus = BusMsg()
    bus.index = msg.index
    bus.goal.CopyFrom(pose_stamped_to_bus(msg.goal))
    bus.error_code = msg.error_code
    return bus


def missed_waypoint_to_ros(bus):
    from nav2_msgs.msg import MissedWaypoint as RosMsg

    out = RosMsg()
    out.index = bus.index
    out.goal = pose_stamped_to_ros(bus.goal)
    out.error_code = bus.error_code
    return out


class Nav2MsgsMissedWaypointMapper:
    def ros_msg_type(self):
        from nav2_msgs.msg import MissedWaypoint as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return missed_waypoint_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import MissedWaypoint as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return missed_waypoint_to_ros(bus)
