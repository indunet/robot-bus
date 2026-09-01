"""Generated mapper for `foxglove_msgs/msg/VoxelGrid`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.packed_element_field import packed_element_field_to_bus, packed_element_field_to_ros

def voxel_grid_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import VoxelGrid as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.frame_id = str(msg.frame_id)
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.row_count = msg.row_count
    bus.column_count = msg.column_count
    bus.cell_size.CopyFrom(vector3_to_bus(msg.cell_size))
    bus.slice_stride = msg.slice_stride
    bus.row_stride = msg.row_stride
    bus.cell_stride = msg.cell_stride
    bus.fields.extend([packed_element_field_to_bus(x) for x in msg.fields])
    bus.data = bytes(msg.data)
    return bus


def voxel_grid_to_ros(bus):
    from foxglove_msgs.msg import VoxelGrid as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.frame_id = str(bus.frame_id)
    out.pose = pose_to_ros(bus.pose)
    out.row_count = bus.row_count
    out.column_count = bus.column_count
    out.cell_size = vector3_to_ros(bus.cell_size)
    out.slice_stride = bus.slice_stride
    out.row_stride = bus.row_stride
    out.cell_stride = bus.cell_stride
    out.fields = [packed_element_field_to_ros(x) for x in bus.fields]
    out.data = bytes(bus.data)
    return out


class FoxgloveMsgsVoxelGridMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import VoxelGrid as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return voxel_grid_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import VoxelGrid as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return voxel_grid_to_ros(bus)
