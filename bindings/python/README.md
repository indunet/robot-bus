# robot-bus (Python)

Python bindings for the robot-bus message bus: `Node`, executors, and topic, service, and action clients, plus the embedded broker (`RobotBusBroker`, `run_broker`).

The wheel embeds the broker, WebSocket server, Web console, and the TANK demo. Message types are generated protobuf classes, for example `from robot_bus.sensor_msgs.msg.v1 import Imu`.

Source, documentation, and the Rust crates (`robot-bus`, `robot-bus-msgs`) live at https://github.com/indunet/robot-bus.
