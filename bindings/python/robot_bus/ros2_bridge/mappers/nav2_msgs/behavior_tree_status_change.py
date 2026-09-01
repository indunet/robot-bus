"""Generated mapper for `nav2_msgs/msg/BehaviorTreeStatusChange`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros

def behavior_tree_status_change_to_bus(msg):
    from robot_bus.nav2_msgs.msg.v1 import BehaviorTreeStatusChange as BusMsg

    bus = BusMsg()
    bus.timestamp.CopyFrom(time_to_bus(msg.timestamp))
    bus.node_name = str(msg.node_name)
    bus.previous_status = str(msg.previous_status)
    bus.current_status = str(msg.current_status)
    return bus


def behavior_tree_status_change_to_ros(bus):
    from nav2_msgs.msg import BehaviorTreeStatusChange as RosMsg

    out = RosMsg()
    out.timestamp = time_to_ros(bus.timestamp)
    out.node_name = str(bus.node_name)
    out.previous_status = str(bus.previous_status)
    out.current_status = str(bus.current_status)
    return out


class Nav2MsgsBehaviorTreeStatusChangeMapper:
    def type_name(self) -> str:
        return "nav2_msgs/msg/BehaviorTreeStatusChange"

    def ros_msg_type(self):
        from nav2_msgs.msg import BehaviorTreeStatusChange as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return behavior_tree_status_change_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.nav2_msgs.msg.v1 import BehaviorTreeStatusChange as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return behavior_tree_status_change_to_ros(bus)
