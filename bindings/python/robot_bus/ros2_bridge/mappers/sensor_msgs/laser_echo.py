"""Generated mapper for `sensor_msgs/msg/LaserEcho`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def laser_echo_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import LaserEcho as BusMsg

    bus = BusMsg()
    bus.echoes.extend(list(msg.echoes))
    return bus


def laser_echo_to_ros(bus):
    from sensor_msgs.msg import LaserEcho as RosMsg

    out = RosMsg()
    out.echoes = list(bus.echoes)
    return out


class SensorMsgsLaserEchoMapper:
    def ros_msg_type(self):
        from sensor_msgs.msg import LaserEcho as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return laser_echo_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import LaserEcho as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return laser_echo_to_ros(bus)
