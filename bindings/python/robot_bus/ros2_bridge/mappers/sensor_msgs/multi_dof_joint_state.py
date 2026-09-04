"""Generated mapper for `sensor_msgs/msg/MultiDOFJointState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform import transform_to_bus, transform_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.wrench import wrench_to_bus, wrench_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.sensor_msgs.msg.v1 import MultiDOFJointState as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def multi_dof_joint_state_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.joint_names.extend([str(x) for x in msg.joint_names])
    bus.transforms.extend([transform_to_bus(x) for x in msg.transforms])
    bus.twist.extend([twist_to_bus(x) for x in msg.twist])
    bus.wrench.extend([wrench_to_bus(x) for x in msg.wrench])
    return bus


def multi_dof_joint_state_to_ros(bus):
    from sensor_msgs.msg import MultiDOFJointState as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.joint_names = [str(x) for x in bus.joint_names]
    out.transforms = [transform_to_ros(x) for x in bus.transforms]
    out.twist = [twist_to_ros(x) for x in bus.twist]
    out.wrench = [wrench_to_ros(x) for x in bus.wrench]
    return out


class SensorMsgsMultiDofJointStateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from sensor_msgs.msg import MultiDOFJointState as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return multi_dof_joint_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return multi_dof_joint_state_to_ros(bus)
