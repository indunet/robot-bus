"""Generated mapper for `action_msgs/msg/GoalStatus`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.action_msgs.goal_info import goal_info_to_bus, goal_info_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.action_msgs.msg.v1 import GoalStatus as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def goal_status_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from action_msgs.msg import GoalStatus as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return goal_status_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return goal_status_to_ros(bus)
