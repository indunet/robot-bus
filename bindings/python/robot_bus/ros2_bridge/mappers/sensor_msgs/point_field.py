"""Generated mapper for `sensor_msgs/msg/PointField`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def point_field_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import PointField as BusMsg

    bus = BusMsg()
    bus.name = str(msg.name)
    bus.offset = msg.offset
    bus.datatype = int(msg.datatype)
    bus.count = msg.count
    return bus


def point_field_to_ros(bus):
    from sensor_msgs.msg import PointField as RosMsg

    out = RosMsg()
    out.name = str(bus.name)
    out.offset = bus.offset
    out.datatype = int(bus.datatype)
    out.count = bus.count
    return out


class SensorMsgsPointFieldMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/PointField"

    def ros_msg_type(self):
        from sensor_msgs.msg import PointField as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point_field_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import PointField as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_field_to_ros(bus)
