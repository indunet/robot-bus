"""Built-in and duck-typed ROS 2 ↔ robot-bus mappers."""

from robot_bus.ros2_bridge.mappers.fibonacci import FibonacciActionMapper
from robot_bus.ros2_bridge.mappers.image import SensorMsgsImageMapper
from robot_bus.ros2_bridge.mappers.set_bool import SetBoolServiceMapper
from robot_bus.ros2_bridge.mappers.string import StdMsgsStringMapper
from robot_bus.ros2_bridge.mappers.trigger import TriggerServiceMapper

__all__ = [
    "FibonacciActionMapper",
    "SensorMsgsImageMapper",
    "SetBoolServiceMapper",
    "StdMsgsStringMapper",
    "TriggerServiceMapper",
]
