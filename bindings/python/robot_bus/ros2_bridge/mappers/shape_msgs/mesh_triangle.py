"""Generated mapper for `shape_msgs/msg/MeshTriangle`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def mesh_triangle_to_bus(msg):
    from robot_bus.shape_msgs.msg.v1 import MeshTriangle as BusMsg

    bus = BusMsg()
    bus.vertex_indices.extend(list(msg.vertex_indices))
    return bus


def mesh_triangle_to_ros(bus):
    from shape_msgs.msg import MeshTriangle as RosMsg

    out = RosMsg()
    out.vertex_indices = list(bus.vertex_indices)
    return out


class ShapeMsgsMeshTriangleMapper:
    def ros_msg_type(self):
        from shape_msgs.msg import MeshTriangle as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return mesh_triangle_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.shape_msgs.msg.v1 import MeshTriangle as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return mesh_triangle_to_ros(bus)
