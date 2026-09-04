"""Generated mapper for `visualization_msgs/msg/MenuEntry`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.visualization_msgs.msg.v1 import MenuEntry as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def menu_entry_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from visualization_msgs.msg import MenuEntry as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return menu_entry_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return menu_entry_to_ros(bus)
