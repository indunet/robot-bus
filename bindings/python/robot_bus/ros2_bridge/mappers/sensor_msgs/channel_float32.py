"""Generated mapper for `sensor_msgs/msg/ChannelFloat32`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import ChannelFloat32 as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def channel_float32_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import ChannelFloat32 as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return channel_float32_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return channel_float32_to_ros(bus)
