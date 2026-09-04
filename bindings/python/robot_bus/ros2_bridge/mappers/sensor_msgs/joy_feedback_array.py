"""Generated mapper for `sensor_msgs/msg/JoyFeedbackArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.sensor_msgs.joy_feedback import joy_feedback_to_bus, joy_feedback_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import JoyFeedbackArray as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joy_feedback_array_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.array.extend([joy_feedback_to_bus(x) for x in msg.array])
    return bus


def joy_feedback_array_to_ros(bus):
    from sensor_msgs.msg import JoyFeedbackArray as RosMsg

    out = RosMsg()
    out.array = [joy_feedback_to_ros(x) for x in bus.array]
    return out


class SensorMsgsJoyFeedbackArrayMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import JoyFeedbackArray as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joy_feedback_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joy_feedback_array_to_ros(bus)
