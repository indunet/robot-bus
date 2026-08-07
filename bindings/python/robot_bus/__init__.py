"""robot_bus — ZeroMQ message bus SDK and ROS-style protobuf message types.

Message types live under this package namespace, for example::

    from robot_bus.sensor_msgs.msg.v1 import Imu

Runtime APIs (``Node``, ``Publisher``, …) come from the Rust extension
``robot_bus._native`` when the wheel / maturin build is installed.

Typed pub/sub (and service/action) are pure-Python wrappers: pass a protobuf
message class to ``create_publisher`` / ``create_subscription`` (etc.) to get
automatic SerializeToString / ParseFromString around the raw-bytes native API.
"""

from __future__ import annotations

try:
    from robot_bus._native import (
        ActionClient,
        ActionGoalHandle,
        CallbackGroup,
        CallbackGroupType,
        Context,
        MultiThreadedExecutor,
        Node,
        Publisher,
        RobotBusBroker,
        ServiceClient,
        ShutdownHandle,
        SingleThreadedExecutor,
        Subscriber,
        TimerHandle,
        TopicPublisher,
        __version__,
        message_xpub_endpoint,
        message_xsub_endpoint,
        run_broker,
    )
except ImportError:  # pragma: no cover - msgs-only / docs import without extension
    ActionClient = None  # type: ignore[misc, assignment]
    ActionGoalHandle = None  # type: ignore[misc, assignment]
    CallbackGroup = None  # type: ignore[misc, assignment]
    CallbackGroupType = None  # type: ignore[misc, assignment]
    Context = None  # type: ignore[misc, assignment]
    MultiThreadedExecutor = None  # type: ignore[misc, assignment]
    Node = None  # type: ignore[misc, assignment]
    Publisher = None  # type: ignore[misc, assignment]
    RobotBusBroker = None  # type: ignore[misc, assignment]
    ServiceClient = None  # type: ignore[misc, assignment]
    ShutdownHandle = None  # type: ignore[misc, assignment]
    SingleThreadedExecutor = None  # type: ignore[misc, assignment]
    Subscriber = None  # type: ignore[misc, assignment]
    TimerHandle = None  # type: ignore[misc, assignment]
    TopicPublisher = None  # type: ignore[misc, assignment]
    __version__ = None  # type: ignore[misc, assignment]
    message_xpub_endpoint = None  # type: ignore[misc, assignment]
    message_xsub_endpoint = None  # type: ignore[misc, assignment]
    run_broker = None  # type: ignore[misc, assignment]
else:
    from robot_bus._typed import install_typed_node_api

    install_typed_node_api(Node)

from robot_bus._typed import (
    TypedActionClient,
    TypedActionGoalHandle,
    TypedServiceClient,
    TypedTopicPublisher,
)

__all__ = [
    "ActionClient",
    "ActionGoalHandle",
    "CallbackGroup",
    "CallbackGroupType",
    "Context",
    "MultiThreadedExecutor",
    "Node",
    "Publisher",
    "RobotBusBroker",
    "ServiceClient",
    "ShutdownHandle",
    "SingleThreadedExecutor",
    "Subscriber",
    "TimerHandle",
    "TopicPublisher",
    "TypedActionClient",
    "TypedActionGoalHandle",
    "TypedServiceClient",
    "TypedTopicPublisher",
    "__version__",
    "message_xpub_endpoint",
    "message_xsub_endpoint",
    "run_broker",
]
