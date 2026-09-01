"""Generated mapper for `visualization_msgs/msg/Marker`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import pose_to_bus, pose_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import vector3_to_bus, vector3_to_ros
from robot_bus.ros2_bridge.mappers.std_msgs.color_rgba import color_rgba_to_bus, color_rgba_to_ros
from robot_bus.ros2_bridge.mappers.builtin_interfaces.duration import duration_to_bus, duration_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.compressed_image import compressed_image_to_bus, compressed_image_to_ros
from robot_bus.ros2_bridge.mappers.visualization_msgs.uv_coordinate import uv_coordinate_to_bus, uv_coordinate_to_ros
from robot_bus.ros2_bridge.mappers.visualization_msgs.mesh_file import mesh_file_to_bus, mesh_file_to_ros

def marker_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import Marker as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.ns = str(msg.ns)
    bus.id = msg.id
    bus.type = msg.type
    bus.action = msg.action
    bus.pose.CopyFrom(pose_to_bus(msg.pose))
    bus.scale.CopyFrom(vector3_to_bus(msg.scale))
    bus.color.CopyFrom(color_rgba_to_bus(msg.color))
    bus.lifetime.CopyFrom(duration_to_bus(msg.lifetime))
    bus.frame_locked = msg.frame_locked
    bus.points.extend([point_to_bus(x) for x in msg.points])
    bus.colors.extend([color_rgba_to_bus(x) for x in msg.colors])
    bus.texture_resource = str(msg.texture_resource)
    bus.texture.CopyFrom(compressed_image_to_bus(msg.texture))
    bus.uv_coordinates.extend([uv_coordinate_to_bus(x) for x in msg.uv_coordinates])
    bus.text = str(msg.text)
    bus.mesh_resource = str(msg.mesh_resource)
    bus.mesh_file.CopyFrom(mesh_file_to_bus(msg.mesh_file))
    bus.mesh_use_embedded_materials = msg.mesh_use_embedded_materials
    return bus


def marker_to_ros(bus):
    from visualization_msgs.msg import Marker as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.ns = str(bus.ns)
    out.id = bus.id
    out.type = bus.type
    out.action = bus.action
    out.pose = pose_to_ros(bus.pose)
    out.scale = vector3_to_ros(bus.scale)
    out.color = color_rgba_to_ros(bus.color)
    out.lifetime = duration_to_ros(bus.lifetime)
    out.frame_locked = bus.frame_locked
    out.points = [point_to_ros(x) for x in bus.points]
    out.colors = [color_rgba_to_ros(x) for x in bus.colors]
    out.texture_resource = str(bus.texture_resource)
    out.texture = compressed_image_to_ros(bus.texture)
    out.uv_coordinates = [uv_coordinate_to_ros(x) for x in bus.uv_coordinates]
    out.text = str(bus.text)
    out.mesh_resource = str(bus.mesh_resource)
    out.mesh_file = mesh_file_to_ros(bus.mesh_file)
    out.mesh_use_embedded_materials = bus.mesh_use_embedded_materials
    return out


class VisualizationMsgsMarkerMapper:
    def type_name(self) -> str:
        return "visualization_msgs/msg/Marker"

    def ros_msg_type(self):
        from visualization_msgs.msg import Marker as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return marker_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import Marker as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return marker_to_ros(bus)
