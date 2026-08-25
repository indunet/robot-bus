"""Builtin: `sensor_msgs/msg/Image` ↔ bus `sensor_msgs.msg.v1.Image`."""

from __future__ import annotations


class SensorMsgsImageMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/Image"

    def ros_msg_type(self):
        from sensor_msgs.msg import Image as RosImage

        return RosImage

    def ros_to_bus(self, msg) -> bytes:
        from robot_bus.sensor_msgs.msg.v1 import Image as BusImage

        bus = BusImage()
        bus.header.frame_id = msg.header.frame_id
        bus.header.stamp.sec = int(msg.header.stamp.sec)
        bus.header.stamp.nanosec = int(msg.header.stamp.nanosec)
        bus.height = int(msg.height)
        bus.width = int(msg.width)
        bus.encoding = str(msg.encoding)
        bus.is_bigendian = bool(msg.is_bigendian)
        bus.step = int(msg.step)
        bus.data = bytes(msg.data)
        return bus.SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import Image as BusImage
        from sensor_msgs.msg import Image as RosImage

        bus = BusImage()
        bus.ParseFromString(payload)
        out = RosImage()
        out.header.frame_id = bus.header.frame_id
        out.header.stamp.sec = bus.header.stamp.sec
        out.header.stamp.nanosec = bus.header.stamp.nanosec
        out.height = bus.height
        out.width = bus.width
        out.encoding = bus.encoding
        out.is_bigendian = 1 if bus.is_bigendian else 0
        out.step = bus.step
        import array

        out.data = array.array("B", bus.data)
        return out
