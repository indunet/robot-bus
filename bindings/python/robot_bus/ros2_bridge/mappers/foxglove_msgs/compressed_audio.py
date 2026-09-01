"""Generated mapper for `foxglove_msgs/msg/CompressedAudio`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def compressed_audio_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import CompressedAudio as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.data = bytes(msg.data)
    bus.format = str(msg.format)
    return bus


def compressed_audio_to_ros(bus):
    from foxglove_msgs.msg import CompressedAudio as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.data = bytes(bus.data)
    out.format = str(bus.format)
    return out


class FoxgloveMsgsCompressedAudioMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/CompressedAudio"

    def ros_msg_type(self):
        from foxglove_msgs.msg import CompressedAudio as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return compressed_audio_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import CompressedAudio as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return compressed_audio_to_ros(bus)
