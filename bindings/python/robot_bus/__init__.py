"""robot_bus — ZeroMQ message bus SDK and ROS-style protobuf message types.

Message types live under this package namespace, for example::

    from robot_bus.sensor_msgs.msg.v1 import Imu

Runtime APIs (``Node``, ``Publisher``, …) come from the Rust extension
``robot_bus._native`` when the wheel / maturin build is installed.

Typed pub/sub (and service/action) are pure-Python wrappers: pass a protobuf
message class to ``create_publisher`` / ``create_subscription`` (etc.) to get
automatic SerializeToString / ParseFromString around the raw-bytes native API.

``Ros2Bridge`` is only present when the extension was built with the ``ros2``
feature (see ``ros2_available()`` / docs/ros2-bridge.md).
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
        ros2_available,
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

    def ros2_available() -> bool:
        return False
else:
    from robot_bus._typed import install_typed_node_api

    install_typed_node_api(Node)

# Optional ROS 2 bridge symbols (missing unless built with `--features …,ros2`).
try:
    from robot_bus._native import (
        Direction,
        DynMsg,
        FibonacciActionMapper,
        Ros2Bridge,
        Ros2BridgeAction,
        Ros2BridgeBuilder,
        Ros2BridgeRoute,
        Ros2BridgeService,
        SensorMsgsImageMapper,
        SensorMsgsImuMapper,
        SetBoolServiceMapper,
        StdMsgsStringMapper,
        TriggerServiceMapper,
    )
except ImportError:  # pragma: no cover
    Direction = None  # type: ignore[misc, assignment]
    DynMsg = None  # type: ignore[misc, assignment]
    FibonacciActionMapper = None  # type: ignore[misc, assignment]
    Ros2Bridge = None  # type: ignore[misc, assignment]
    Ros2BridgeAction = None  # type: ignore[misc, assignment]
    Ros2BridgeBuilder = None  # type: ignore[misc, assignment]
    Ros2BridgeRoute = None  # type: ignore[misc, assignment]
    Ros2BridgeService = None  # type: ignore[misc, assignment]
    SensorMsgsImageMapper = None  # type: ignore[misc, assignment]
    SensorMsgsImuMapper = None  # type: ignore[misc, assignment]
    SetBoolServiceMapper = None  # type: ignore[misc, assignment]
    StdMsgsStringMapper = None  # type: ignore[misc, assignment]
    TriggerServiceMapper = None  # type: ignore[misc, assignment]

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
    "Direction",
    "DynMsg",
    "FibonacciActionMapper",
    "MultiThreadedExecutor",
    "Node",
    "Publisher",
    "RobotBusBroker",
    "Ros2Bridge",
    "Ros2BridgeAction",
    "Ros2BridgeBuilder",
    "Ros2BridgeRoute",
    "Ros2BridgeService",
    "SensorMsgsImageMapper",
    "SensorMsgsImuMapper",
    "ServiceClient",
    "SetBoolServiceMapper",
    "ShutdownHandle",
    "SingleThreadedExecutor",
    "StdMsgsStringMapper",
    "Subscriber",
    "TimerHandle",
    "TopicPublisher",
    "TriggerServiceMapper",
    "TypedActionClient",
    "TypedActionGoalHandle",
    "TypedServiceClient",
    "TypedTopicPublisher",
    "__version__",
    "message_xpub_endpoint",
    "message_xsub_endpoint",
    "ros2_available",
    "run_broker",
]
