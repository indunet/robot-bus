"""Generated mapper for `sensor_msgs/msg/RegionOfInterest`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import RegionOfInterest as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def region_of_interest_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.x_offset = msg.x_offset
    bus.y_offset = msg.y_offset
    bus.height = msg.height
    bus.width = msg.width
    bus.do_rectify = msg.do_rectify
    return bus


def region_of_interest_to_ros(bus):
    from sensor_msgs.msg import RegionOfInterest as RosMsg

    out = RosMsg()
    out.x_offset = bus.x_offset
    out.y_offset = bus.y_offset
    out.height = bus.height
    out.width = bus.width
    out.do_rectify = bus.do_rectify
    return out


class SensorMsgsRegionOfInterestMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import RegionOfInterest as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return region_of_interest_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return region_of_interest_to_ros(bus)
