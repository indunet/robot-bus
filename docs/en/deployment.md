English | [中文](../zh/deployment.md)

# Deployment and troubleshooting

First run the [three README quick starts](../../README.md#1-quick-start). Then configure connections for your deployment. See [support status](support.md) for implementation and live ROS verification boundaries.

## Choose processes and transport

| Scenario | Suggested setup | Requirement |
| --- | --- | --- |
| One application owns the bus lifecycle | Embed `RobotBusBroker.start()` | Keep the broker alive until business nodes stop; do not start a broker in every node process |
| Multiple processes or a daemon | One standalone broker with connecting clients | Use a service manager for lifecycle and logs; clients wait for readiness |
| Native SDK across processes/hosts | TCP plus HTTP discover | Default ZMQ data ports are dynamically assigned; opening only HTTP is insufficient |
| Same-process communication | inproc with shared `Context` | Broker and nodes must share a Context; inproc cannot cross processes |
| Browser or clients needing one entry point | WS `/ws-rpc` | Shares the HTTP port; can call services/actions, but cannot register servers |

Embedded lifecycle example (runnable after installing the Python package; it sends no business messages):

```python
import robot_bus

ctx = robot_bus.Context()
with robot_bus.RobotBusBroker.start(context=ctx, api_listen="127.0.0.1:0"):
    node = robot_bus.Node.inproc_with_context(ctx, "application")
    try:
        # Register publishers/subscriptions/services/actions before starting callbacks.
        node.start()
        print("broker and node started")
    finally:
        node.shutdown()
        node.stop()
        node.wait()
# Exiting with stops the broker. Real applications wait for business work inside with.
```

## Local startup and checks

```bash
python -m robot_bus.broker --api-listen 127.0.0.1:15560 --tcp-only
```

In another terminal:

```bash
curl --fail http://127.0.0.1:15560/api/v1/discover
```

Expect JSON containing the broker's advertised connection information. Open `http://127.0.0.1:15560` for the console. HTTP success only establishes API reachability; native clients also need the returned data endpoints. `0.0.0.0` binds all interfaces and is not a client destination.

The shared API defaults to port `15560`. If the requested port is in use, the broker tries up to 32 following ports in order; `:0` lets the OS allocate a port. WebSocket, HTTP discovery, and the console share the resulting port. Use the actual port from the startup log or the SDK's `api_listen` for clients and container port mappings; clients do not scan for it.

## Multiple hosts and fixed ports

This example uses `10.0.0.2` as the broker's reachable address; replace it with your address. These ports are explicitly configured here, not broker defaults.

```bash
python -m robot_bus.broker --tcp-only \
  --api-listen 0.0.0.0:15560 \
  --message-xsub-bind tcp://0.0.0.0:15580 \
  --message-xpub-bind tcp://0.0.0.0:15581 \
  --service-frontend-bind tcp://0.0.0.0:15662 \
  --service-backend-bind tcp://0.0.0.0:15663 \
  --action-frontend-bind tcp://0.0.0.0:15664 \
  --action-backend-bind tcp://0.0.0.0:15665 --no-tank
```

| Example port | Purpose | Connecting participants |
| --- | --- | --- |
| 15560 | HTTP discover, console, WS | Discovery and WS clients |
| 15580 / 15581 | Topic XSUB / XPUB | Native publishers / subscribers |
| 15662 / 15663 | Service frontend / backend | Native clients / servers |
| 15664 / 15665 | Action frontend / backend | Native clients / servers |

Native clients and `--peer` use the IP address or DNS name they contacted at `/discover` and take the six numeric data ports from the response. The broker does not infer or report a host by default. From the client machine, check that the data ports are reachable on the discovery host. Upgrade discovery clients and brokers together; the previous endpoint-string response format is not supported.

The JSON fields are `messageXsubPort`, `messageXpubPort`, `serviceFrontendPort`, `serviceBackendPort`, `actionFrontendPort`, and `actionBackendPort`.

If the HTTP entry point and native TCP listeners use different hosts (for example with a reverse proxy, NAT, or an interface-specific bind), set `--advertise-host HOST`. The response then includes `advertiseHost` for TCP, without changing listening interfaces or creating port mappings. Published NAT or container ports must match the data ports in the response.

Native Python node:

```python
import robot_bus
node = robot_bus.Node.discover(
    "remote", transport="tcp", api_url="http://10.0.0.2:15560",
)
```

WS clients use `robot_bus.Node.ws_at("remote", "http://10.0.0.2:15560")`. HTTPS pages need a `wss://` entry point providing TLS termination and WebSocket upgrade. Restrict bus/control interfaces with network access controls and add proxy authentication where needed; CORS is not authentication. `--no-console` disables console components but leaves WS/discover available and is not access control.

## Readiness, callbacks and shutdown

- Successful `Node(...)` construction does not establish a connection. Use `wait_for_broker(timeout=5.0)` or explicit `Node.discover(...)`.
- After registering subscriptions/servers, run `spin()`, `spin_once()` or native Node `start()` to drive callbacks. Do not move a Python Node to another Python thread; use `start()` for background execution.
- Use `wait_for_service` / `wait_for_action_server` before requests. These helpers depend on broker monitoring information and are best-effort; check that minimal builds include that capability.
- Before blocking for results on the main thread, the server must run in the background or another process. Calling `spin()` after `call()` / `result()` cannot resolve that wait.
- Stop nodes and wait for background execution before exiting the broker context. Running user callbacks are not forcibly terminated and must cooperate with shutdown.
- Reconnection after broker restart does not restore in-flight RPCs. Timeout does not revoke an operation; design idempotency keys or result queries before retrying side effects.

## Troubleshoot by symptom

| Symptom | First check | Next action |
| --- | --- | --- |
| API is not on port 15560 | Actual port in the startup log | Connect using the actual port; only reuse an existing broker if intended |
| Console works; native node cannot connect | Discovery host and data ports, explicit override, port mappings | Allow the TCP ports; set `--advertise-host` only when the data host differs |
| No topic output | Matching topic/type and active callback loop | Use repeated publication to accommodate asynchronous subscriptions; inspect Topics/Topology |
| Service/action timeout | Server process and `start()`/`spin()`, route names, available workers | Run standard examples first; inspect blocking business callbacks rather than retrying indefinitely |
| No inproc messages | Same process and shared Context | Pair `Node.inproc_with_context` with `RobotBusBroker.start(context=ctx)` |
| WS disconnect or unknown opcode 5 | Broker/SDK compatibility | Upgrade broker first; do not mix V2/V3; check proxy WebSocket upgrade |
| Slow subscriptions or drops | Pending/dropped counters and callback duration | Size queues for load; WS KeepLast controls server pending messages only, native HWM does not guarantee oldest-message replacement |
| ROS bridge unavailable or fails to compile | Sourced environment, rclpy/rclcpp, Rust IDL/typesupport | Read [support status](support.md) and bridge troubleshooting; retain environment details and full errors |

When reporting an issue, include version/commit, OS/architecture, language, transport, broker/node commands, expected and actual output. For ROS add distribution, RMW and source/overlay details. Remove credentials from logs. See [build profiles](build-profiles.md) and [Rust API](rust-api.md) for build, queue and error semantics.
