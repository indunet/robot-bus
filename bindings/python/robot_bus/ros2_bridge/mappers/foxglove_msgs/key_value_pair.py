"""Generated mapper for `foxglove_msgs/msg/KeyValuePair`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import KeyValuePair as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def key_value_pair_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.key = str(msg.key)
    bus.value = str(msg.value)
    return bus


def key_value_pair_to_ros(bus):
    from foxglove_msgs.msg import KeyValuePair as RosMsg

    out = RosMsg()
    out.key = str(bus.key)
    out.value = str(bus.value)
    return out


class FoxgloveMsgsKeyValuePairMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import KeyValuePair as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return key_value_pair_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return key_value_pair_to_ros(bus)
