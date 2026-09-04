"""Generated mapper for `sensor_msgs/msg/JoyFeedback`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import JoyFeedback as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joy_feedback_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.type = msg.type
    bus.id = msg.id
    bus.intensity = msg.intensity
    return bus


def joy_feedback_to_ros(bus):
    from sensor_msgs.msg import JoyFeedback as RosMsg

    out = RosMsg()
    out.type = bus.type
    out.id = bus.id
    out.intensity = bus.intensity
    return out


class SensorMsgsJoyFeedbackMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import JoyFeedback as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joy_feedback_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joy_feedback_to_ros(bus)
