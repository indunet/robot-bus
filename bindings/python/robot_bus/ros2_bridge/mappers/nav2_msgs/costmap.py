"""Generated mapper for `nav2_msgs/msg/Costmap`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.nav2_msgs.costmap_meta_data import costmap_meta_data_to_bus, costmap_meta_data_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import Costmap as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def costmap_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.metadata.CopyFrom(costmap_meta_data_to_bus(msg.metadata))
    bus.data = _convert.i8_seq_to_bytes(msg.data)
    return bus


def costmap_to_ros(bus):
    from nav2_msgs.msg import Costmap as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.metadata = costmap_meta_data_to_ros(bus.metadata)
    out.data = _convert.bytes_to_i8_seq(bus.data)
    return out


class Nav2MsgsCostmapMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import Costmap as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return costmap_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return costmap_to_ros(bus)
