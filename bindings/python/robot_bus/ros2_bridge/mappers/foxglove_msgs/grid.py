"""Generated mapper for `foxglove_msgs/msg/Grid`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector2 import vector2_to_bus, vector2_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.packed_element_field import packed_element_field_to_bus, packed_element_field_to_ros

def grid_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import Grid as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.column_count = msg.column_count
    bus.cell_size.CopyFrom(vector2_to_bus(msg.cell_size))
    bus.row_stride = msg.row_stride
    bus.cell_stride = msg.cell_stride
    bus.fields.extend([packed_element_field_to_bus(x) for x in msg.fields])
    bus.data = bytes(msg.data)
    return bus


def grid_to_ros(bus):
    from foxglove_msgs.msg import Grid as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.pose = pose_to_ros(bus.pose)
    out.column_count = bus.column_count
    out.cell_size = vector2_to_ros(bus.cell_size)
    out.row_stride = bus.row_stride
    out.cell_stride = bus.cell_stride
    out.fields = [packed_element_field_to_ros(x) for x in bus.fields]
    out.data = bytes(bus.data)
    return out


class FoxgloveMsgsGridMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/Grid"

    def ros_msg_type(self):
        from foxglove_msgs.msg import Grid as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return grid_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import Grid as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return grid_to_ros(bus)
