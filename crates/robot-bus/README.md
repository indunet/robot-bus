# robot-bus

Client SDK for the robot-bus message bus: `Node`, executors, and topic, service, and action clients.

The broker, WebSocket server, console, and `rbus` CLI are the `robot-bus-broker` crate. Generated messages live in `robot-bus-msgs` and are re-exported here.

WebSocket clients need `features = ["ws"]`. Native ZeroMQ is the default.
