"""Generated mapper for `nav2_msgs/msg/BehaviorTreeLog`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros
from robot_bus.ros2_bridge.mappers.nav2_msgs.behavior_tree_status_change import behavior_tree_status_change_to_bus, behavior_tree_status_change_to_ros

def behavior_tree_log_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import BehaviorTreeLog as BusMsg

    bus = BusMsg()
    bus.timestamp.CopyFrom(time_to_bus(msg.timestamp))
    bus.event_log.extend([behavior_tree_status_change_to_bus(x) for x in msg.event_log])
    return bus


def behavior_tree_log_to_ros(bus):
    from nav2_msgs.msg import BehaviorTreeLog as RosMsg

    out = RosMsg()
    out.timestamp = time_to_ros(bus.timestamp)
    out.event_log = [behavior_tree_status_change_to_ros(x) for x in bus.event_log]
    return out


class Nav2MsgsBehaviorTreeLogMapper:
    def ros_msg_type(self):
        from nav2_msgs.msg import BehaviorTreeLog as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return behavior_tree_log_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import BehaviorTreeLog as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return behavior_tree_log_to_ros(bus)
