"""robot_bus — ZeroMQ message bus SDK and ROS-style protobuf message types.

Message types live under this package namespace, for example::

    from robot_bus.sensor_msgs.msg.v1 import Imu

Runtime APIs (``Node``, ``Publisher``, …) come from the Rust extension
``robot_bus._native`` when the wheel / maturin build is installed.
"""

try:
    from robot_bus._native import (
        Node,
        Publisher,
        RobotBusBroker,
        ShutdownHandle,
        Subscriber,
        TimerHandle,
        __version__,
        message_xpub_endpoint,
        message_xsub_endpoint,
        run_broker,
    )
except ImportError:  # pragma: no cover - msgs-only / docs import without extension
    Node = None  # type: ignore[misc, assignment]
    Publisher = None  # type: ignore[misc, assignment]
    RobotBusBroker = None  # type: ignore[misc, assignment]
    ShutdownHandle = None  # type: ignore[misc, assignment]
    Subscriber = None  # type: ignore[misc, assignment]
    TimerHandle = None  # type: ignore[misc, assignment]
    __version__ = None  # type: ignore[misc, assignment]
    message_xpub_endpoint = None  # type: ignore[misc, assignment]
    message_xsub_endpoint = None  # type: ignore[misc, assignment]
    run_broker = None  # type: ignore[misc, assignment]

__all__ = [
    "Node",
    "Publisher",
    "RobotBusBroker",
    "ShutdownHandle",
    "Subscriber",
    "TimerHandle",
    "__version__",
    "message_xpub_endpoint",
    "message_xsub_endpoint",
    "run_broker",
]
