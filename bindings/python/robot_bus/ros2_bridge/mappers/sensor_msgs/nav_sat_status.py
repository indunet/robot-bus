"""Generated mapper for `sensor_msgs/msg/NavSatStatus`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def nav_sat_status_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import NavSatStatus as BusMsg

    bus = BusMsg()
    bus.status = int(msg.status)
    bus.service = int(msg.service)
    return bus


def nav_sat_status_to_ros(bus):
    from sensor_msgs.msg import NavSatStatus as RosMsg

    out = RosMsg()
    out.status = int(bus.status)
    out.service = int(bus.service)
    return out


class SensorMsgsNavSatStatusMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/NavSatStatus"

    def ros_msg_type(self):
        from sensor_msgs.msg import NavSatStatus as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return nav_sat_status_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import NavSatStatus as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return nav_sat_status_to_ros(bus)
