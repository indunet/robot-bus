"""Generated mapper for `foxglove_msgs/msg/LocationFixes`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.location_fix import location_fix_to_bus, location_fix_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import LocationFixes as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def location_fixes_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.fixes.extend([location_fix_to_bus(x) for x in msg.fixes])
    return bus


def location_fixes_to_ros(bus):
    from foxglove_msgs.msg import LocationFixes as RosMsg

    out = RosMsg()
    out.fixes = [location_fix_to_ros(x) for x in bus.fixes]
    return out


class FoxgloveMsgsLocationFixesMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import LocationFixes as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return location_fixes_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return location_fixes_to_ros(bus)
