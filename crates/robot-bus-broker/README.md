# robot-bus-broker

Broker, WebSocket server, embedded console, tank demo, and the `rbus` CLI.

Depend on `robot-bus` as well when the same process creates `Node`s. Client-only programs do not need this crate.

Default features are `ws`, `console`, and `demo-tank`. Use `--no-default-features` to build a smaller broker.
