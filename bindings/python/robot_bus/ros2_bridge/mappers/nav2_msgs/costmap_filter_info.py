"""Generated mapper for `nav2_msgs/msg/CostmapFilterInfo`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.nav2_msgs.msg.v1 import CostmapFilterInfo as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def costmap_filter_info_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.type = msg.type
    bus.filter_mask_topic = str(msg.filter_mask_topic)
    bus.base = msg.base
    bus.multiplier = msg.multiplier
    return bus


def costmap_filter_info_to_ros(bus):
    from nav2_msgs.msg import CostmapFilterInfo as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.type = bus.type
    out.filter_mask_topic = str(bus.filter_mask_topic)
    out.base = bus.base
    out.multiplier = bus.multiplier
    return out


class Nav2MsgsCostmapFilterInfoMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from nav2_msgs.msg import CostmapFilterInfo as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return costmap_filter_info_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return costmap_filter_info_to_ros(bus)
