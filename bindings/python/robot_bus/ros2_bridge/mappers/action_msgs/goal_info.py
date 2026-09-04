"""Generated mapper for `action_msgs/msg/GoalInfo`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.unique_identifier_msgs.uuid import uuid_to_bus, uuid_to_ros
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.action_msgs.msg.v1 import GoalInfo as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def goal_info_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.goal_id.CopyFrom(uuid_to_bus(msg.goal_id))
    bus.stamp.CopyFrom(time_to_bus(msg.stamp))
    return bus


def goal_info_to_ros(bus):
    from action_msgs.msg import GoalInfo as RosMsg

    out = RosMsg()
    out.goal_id = uuid_to_ros(bus.goal_id)
    out.stamp = time_to_ros(bus.stamp)
    return out


class ActionMsgsGoalInfoMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from action_msgs.msg import GoalInfo as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return goal_info_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return goal_info_to_ros(bus)
