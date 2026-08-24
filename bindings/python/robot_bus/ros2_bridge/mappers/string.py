"""Builtin: `std_msgs/msg/String` ↔ bus `std_msgs.msg.v1.String`."""

from __future__ import annotations


class StdMsgsStringMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/String"

    def ros_msg_type(self):
        from std_msgs.msg import String as RosString

        return RosString

    def ros_to_bus(self, msg) -> bytes:
        from robot_bus.std_msgs.msg.v1 import String as BusString

        return BusString(data=str(msg.data)).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import String as BusString
        from std_msgs.msg import String as RosString

        bus = BusString()
        bus.ParseFromString(payload)
        out = RosString()
        out.data = bus.data
        return out
