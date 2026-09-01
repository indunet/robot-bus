"""Generated mapper for `foxglove_msgs/msg/LocationFixes`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.location_fix import location_fix_to_bus, location_fix_to_ros

def location_fixes_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import LocationFixes as BusMsg

    bus = BusMsg()
    bus.fixes.extend([location_fix_to_bus(x) for x in msg.fixes])
    return bus


def location_fixes_to_ros(bus):
    from foxglove_msgs.msg import LocationFixes as RosMsg

    out = RosMsg()
    out.fixes = [location_fix_to_ros(x) for x in bus.fixes]
    return out


class FoxgloveMsgsLocationFixesMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/LocationFixes"

    def ros_msg_type(self):
        from foxglove_msgs.msg import LocationFixes as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return location_fixes_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import LocationFixes as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return location_fixes_to_ros(bus)
