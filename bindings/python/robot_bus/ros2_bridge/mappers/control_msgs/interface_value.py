"""Generated mapper for `control_msgs/msg/InterfaceValue`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import InterfaceValue as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def interface_value_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.interface_names.extend([str(x) for x in msg.interface_names])
    bus.values.extend(list(msg.values))
    return bus


def interface_value_to_ros(bus):
    from control_msgs.msg import InterfaceValue as RosMsg

    out = RosMsg()
    out.interface_names = [str(x) for x in bus.interface_names]
    out.values = list(bus.values)
    return out


class ControlMsgsInterfaceValueMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import InterfaceValue as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return interface_value_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return interface_value_to_ros(bus)
