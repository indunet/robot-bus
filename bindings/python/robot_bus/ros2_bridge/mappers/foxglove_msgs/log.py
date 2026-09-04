"""Generated mapper for `foxglove_msgs/msg/Log`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import Log as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def log_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.level = int(msg.level)
    bus.message = str(msg.message)
    bus.name = str(msg.name)
    bus.file = str(msg.file)
    bus.line = msg.line
    return bus


def log_to_ros(bus):
    from foxglove_msgs.msg import Log as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.level = int(bus.level)
    out.message = str(bus.message)
    out.name = str(bus.name)
    out.file = str(bus.file)
    out.line = bus.line
    return out


class FoxgloveMsgsLogMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import Log as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return log_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return log_to_ros(bus)
