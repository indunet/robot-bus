"""Generated mapper for `foxglove_msgs/msg/Log`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def log_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import Log as BusMsg

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
    def type_name(self) -> str:
        return "foxglove_msgs/msg/Log"

    def ros_msg_type(self):
        from foxglove_msgs.msg import Log as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return log_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import Log as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return log_to_ros(bus)
