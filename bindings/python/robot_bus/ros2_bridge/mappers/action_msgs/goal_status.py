"""Generated mapper for `action_msgs/msg/GoalStatus`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.action_msgs.goal_info import goal_info_to_bus, goal_info_to_ros

def goal_status_to_bus(msg):
    from robot_bus.action_msgs.msg.v1 import GoalStatus as BusMsg

    bus = BusMsg()
    bus.goal_info.CopyFrom(goal_info_to_bus(msg.goal_info))
    bus.status = int(msg.status)
    return bus


def goal_status_to_ros(bus):
    from action_msgs.msg import GoalStatus as RosMsg

    out = RosMsg()
    out.goal_info = goal_info_to_ros(bus.goal_info)
    out.status = int(bus.status)
    return out


class ActionMsgsGoalStatusMapper:
    def ros_msg_type(self):
        from action_msgs.msg import GoalStatus as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return goal_status_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.action_msgs.msg.v1 import GoalStatus as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return goal_status_to_ros(bus)
