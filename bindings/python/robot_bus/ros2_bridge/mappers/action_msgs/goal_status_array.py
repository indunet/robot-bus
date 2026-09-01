"""Generated mapper for `action_msgs/msg/GoalStatusArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.action_msgs.goal_status import goal_status_to_bus, goal_status_to_ros

def goal_status_array_to_bus(msg):
    from robot_bus.action_msgs.msg.v1 import GoalStatusArray as BusMsg

    bus = BusMsg()
    bus.status_list.extend([goal_status_to_bus(x) for x in msg.status_list])
    return bus


def goal_status_array_to_ros(bus):
    from action_msgs.msg import GoalStatusArray as RosMsg

    out = RosMsg()
    out.status_list = [goal_status_to_ros(x) for x in bus.status_list]
    return out


class ActionMsgsGoalStatusArrayMapper:
    def type_name(self) -> str:
        return "action_msgs/msg/GoalStatusArray"

    def ros_msg_type(self):
        from action_msgs.msg import GoalStatusArray as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return goal_status_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.action_msgs.msg.v1 import GoalStatusArray as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return goal_status_array_to_ros(bus)
