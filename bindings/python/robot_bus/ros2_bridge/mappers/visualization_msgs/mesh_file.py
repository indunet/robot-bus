"""Generated mapper for `visualization_msgs/msg/MeshFile`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def mesh_file_to_bus(msg):
    from robot_bus.visualization_msgs.msg.v1 import MeshFile as BusMsg

    bus = BusMsg()
    bus.filename = str(msg.filename)
    bus.data = bytes(msg.data)
    return bus


def mesh_file_to_ros(bus):
    from visualization_msgs.msg import MeshFile as RosMsg

    out = RosMsg()
    out.filename = str(bus.filename)
    out.data = bytes(bus.data)
    return out


class VisualizationMsgsMeshFileMapper:
    def ros_msg_type(self):
        from visualization_msgs.msg import MeshFile as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return mesh_file_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.visualization_msgs.msg.v1 import MeshFile as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return mesh_file_to_ros(bus)
