"""Generated mapper for `shape_msgs/msg/Mesh`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.shape_msgs.mesh_triangle import mesh_triangle_to_bus, mesh_triangle_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

def mesh_to_bus(msg):
    from robot_bus.shape_msgs.msg.v1 import Mesh as BusMsg

    bus = BusMsg()
    bus.triangles.extend([mesh_triangle_to_bus(x) for x in msg.triangles])
    bus.vertices.extend([point_to_bus(x) for x in msg.vertices])
    return bus


def mesh_to_ros(bus):
    from shape_msgs.msg import Mesh as RosMsg

    out = RosMsg()
    out.triangles = [mesh_triangle_to_ros(x) for x in bus.triangles]
    out.vertices = [point_to_ros(x) for x in bus.vertices]
    return out


class ShapeMsgsMeshMapper:
    def type_name(self) -> str:
        return "shape_msgs/msg/Mesh"

    def ros_msg_type(self):
        from shape_msgs.msg import Mesh as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return mesh_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.shape_msgs.msg.v1 import Mesh as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return mesh_to_ros(bus)
