"""Generated mapper for `sensor_msgs/msg/MagneticField`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros

def magnetic_field_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import MagneticField as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.magnetic_field.CopyFrom(vector3_to_bus(msg.magnetic_field))
    bus.magnetic_field_covariance.extend(list(msg.magnetic_field_covariance))
    return bus


def magnetic_field_to_ros(bus):
    from sensor_msgs.msg import MagneticField as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.magnetic_field = vector3_to_ros(bus.magnetic_field)
    out.magnetic_field_covariance = list(bus.magnetic_field_covariance)
    return out


class SensorMsgsMagneticFieldMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/MagneticField"

    def ros_msg_type(self):
        from sensor_msgs.msg import MagneticField as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return magnetic_field_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import MagneticField as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return magnetic_field_to_ros(bus)
