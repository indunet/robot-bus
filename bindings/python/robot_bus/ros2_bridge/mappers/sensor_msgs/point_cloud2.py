"""Generated mapper for `sensor_msgs/msg/PointCloud2`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.point_field import point_field_to_bus, point_field_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import PointCloud2 as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def point_cloud2_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.height = msg.height
    bus.width = msg.width
    bus.fields.extend([point_field_to_bus(x) for x in msg.fields])
    bus.is_bigendian = msg.is_bigendian
    bus.point_step = msg.point_step
    bus.row_step = msg.row_step
    bus.data = bytes(msg.data)
    bus.is_dense = msg.is_dense
    return bus


def point_cloud2_to_ros(bus):
    from sensor_msgs.msg import PointCloud2 as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.height = bus.height
    out.width = bus.width
    out.fields = [point_field_to_ros(x) for x in bus.fields]
    out.is_bigendian = bus.is_bigendian
    out.point_step = bus.point_step
    out.row_step = bus.row_step
    out.data = bytes(bus.data)
    out.is_dense = bus.is_dense
    return out


class SensorMsgsPointCloud2Mapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import PointCloud2 as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return point_cloud2_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_cloud2_to_ros(bus)
