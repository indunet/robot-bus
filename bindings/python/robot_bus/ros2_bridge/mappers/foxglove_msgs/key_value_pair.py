"""Generated mapper for `foxglove_msgs/msg/KeyValuePair`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def key_value_pair_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import KeyValuePair as BusMsg

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
    def ros_msg_type(self):
        from foxglove_msgs.msg import KeyValuePair as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return key_value_pair_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import KeyValuePair as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return key_value_pair_to_ros(bus)
