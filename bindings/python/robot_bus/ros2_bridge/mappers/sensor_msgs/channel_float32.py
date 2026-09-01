"""Generated mapper for `sensor_msgs/msg/ChannelFloat32`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def channel_float32_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import ChannelFloat32 as BusMsg

    bus = BusMsg()
    bus.name = str(msg.name)
    bus.values.extend(list(msg.values))
    return bus


def channel_float32_to_ros(bus):
    from sensor_msgs.msg import ChannelFloat32 as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.values = list(bus.values)
    return out


class SensorMsgsChannelFloat32Mapper:
    def ros_msg_type(self):
        from sensor_msgs.msg import ChannelFloat32 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return channel_float32_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import ChannelFloat32 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return channel_float32_to_ros(bus)
