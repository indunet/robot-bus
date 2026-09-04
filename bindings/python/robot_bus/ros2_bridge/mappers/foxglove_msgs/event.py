"""Generated mapper for `foxglove_msgs/msg/Event`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import Event as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def event_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.start_time = _convert.time_to_timestamp(msg.start_time)
    bus.end_time = _convert.time_to_timestamp(msg.end_time)
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    return bus


def event_to_ros(bus):
    from foxglove_msgs.msg import Event as RosMsg

    out = RosMsg()
    out.start_time = _convert.timestamp_to_time(bus.start_time)
    out.end_time = _convert.timestamp_to_time(bus.end_time)
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    return out


class FoxgloveMsgsEventMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import Event as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return event_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return event_to_ros(bus)
