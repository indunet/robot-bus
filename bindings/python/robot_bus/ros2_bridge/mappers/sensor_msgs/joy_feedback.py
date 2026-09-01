"""Generated mapper for `sensor_msgs/msg/JoyFeedback`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def joy_feedback_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import JoyFeedback as BusMsg

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
    def ros_msg_type(self):
        from sensor_msgs.msg import JoyFeedback as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return joy_feedback_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import JoyFeedback as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return joy_feedback_to_ros(bus)
