"""Generated mapper for `visualization_msgs/msg/MenuEntry`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def menu_entry_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import MenuEntry as BusMsg

    bus = BusMsg()
    bus.id = msg.id
    bus.parent_id = msg.parent_id
    bus.title = str(msg.title)
    bus.command = str(msg.command)
    bus.command_type = msg.command_type
    return bus


def menu_entry_to_ros(bus):
    from visualization_msgs.msg import MenuEntry as RosMsg

    out = RosMsg()
    out.id = bus.id
    out.parent_id = bus.parent_id
    out.title = str(bus.title)
    out.command = str(bus.command)
    out.command_type = bus.command_type
    return out


class VisualizationMsgsMenuEntryMapper:
    def ros_msg_type(self):
        from visualization_msgs.msg import MenuEntry as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return menu_entry_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import MenuEntry as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return menu_entry_to_ros(bus)
