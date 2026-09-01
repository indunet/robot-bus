"""Generated mapper for `foxglove_msgs/msg/RawAudio`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def raw_audio_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import RawAudio as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.data = bytes(msg.data)
    bus.format = str(msg.format)
    bus.sample_rate = msg.sample_rate
    bus.number_of_channels = msg.number_of_channels
    return bus


def raw_audio_to_ros(bus):
    from foxglove_msgs.msg import RawAudio as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.data = bytes(bus.data)
    out.format = str(bus.format)
    out.sample_rate = bus.sample_rate
    out.number_of_channels = bus.number_of_channels
    return out


class FoxgloveMsgsRawAudioMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import RawAudio as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return raw_audio_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import RawAudio as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return raw_audio_to_ros(bus)
