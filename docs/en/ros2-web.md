[中文](../zh/ros2-web.md) | English

# Browser → Robot Bus → real ROS2

The [bridge lab](../../examples/ros2_web/README.md) provides a runnable acceptance
scenario for **ROS2 Humble, Python/rclpy, Fast DDS and Chromium**. It builds the
broker and TypeScript/Python SDKs from the current checkout. A separate process
runs the ROS node; communication with the bridge uses real DDS entities.

```text
Browser TypeScript SDK ←WebSocket→ Robot Bus broker ←TCP→ Python bridge ←DDS→ ROS peer
```

## Start

From the repository root:

```bash
docker compose -f examples/ros2_web/compose.yaml up --build
```

Open <http://127.0.0.1:8080>. Send a command and observe the value reported by ROS;
click Reset to call a ROS service; start a task to see feedback and a result.
Use 30 terms to allow time to cancel. The failure-path panel exercises rejection,
ROS abort and a bridge deadline. Topic status is sent at 10 Hz.

This is a standalone example UI; the broker builds with `ws,console-api` and does
not need the embedded Next.js console. Compose exposes only loopback ports 8080
and 15560. DDS stays inside the container (`ROS_LOCALHOST_ONLY=1`, domain 73).

## Routes and assertions

All names below use the `/ros2_web` prefix. Topic directions describe the sample
flow; RPC directions describe where the initiating client lives.

| Bus route | ROS route | Direction / contract | Acceptance assertion |
| --- | --- | --- | --- |
| `/status` | `/status` | ROS → bus, `std_msgs/String` | Increasing ROS sequence reaches browser |
| `/command` | `/command` | Bus → ROS, `std_msgs/String` | Unique browser command appears in ROS status |
| `/reset` | `/reset` | Bus → ROS, `std_srvs/Trigger` | Exact response and incremented ROS reset count |
| `/slow` | `/slow` | Bus → ROS, `std_srvs/Trigger` | Typed `timeout`; subsequent reset works |
| `/fibonacci` | `/fibonacci` | Bus → ROS, `example_interfaces/Fibonacci` | Feedback, exact result, rejection, abort, cancellation |
| `/deadline` | `/fibonacci` | Bus → ROS, same action | Typed `timeout` and ROS confirms cancellation |

The demo computes exactly `order` Fibonacci terms. Orders outside 1–30 are
rejected; order 13 deliberately aborts. Normal actions have a 10-second bridge
budget; the deadline route has 0.4 seconds. The slow service takes 2 seconds but
its bridge budget is 0.4 seconds. A timeout is **not** rollback; the ROS service
still completes. Topic establishment is asynchronous, so the test may repeat the
same idempotent command within a bounded window. RPCs are not automatically retried.

## Automated acceptance and evidence

The [test runner](../../examples/ros2_web/test.sh) starts the backend and executes
eight Playwright tests in Chromium with bounded waits and no skipped dependencies.
The [CI workflow](../../.github/workflows/ros2-web.yml) builds the same Dockerfile
on Ubuntu 22.04 and preserves evidence even after test failure.

```bash
docker build -f examples/ros2_web/Dockerfile -t robot-bus-ros2-web:test .
docker run --name ros2-web-acceptance --init \
  -e ROBOT_BUS_TEST_COMMIT="$(git rev-parse HEAD)" \
  robot-bus-ros2-web:test bash test.sh
# Also run after test failure:
docker cp ros2-web-acceptance:/workspace/examples/ros2_web/test-results ./ros2-web-results
docker rm ros2-web-acceptance
```

Evidence contains `environment.log` (commit, OS/architecture, ROS/RMW settings and
tool versions), `backend.log`, JUnit results and browser traces/screenshots on
failure. A configured workflow is not proof that a particular commit passed;
inspect its run and artifacts. Docker tags and apt repositories can move; the
environment record describes the versions actually used.

## Local development with sourced Humble

Use Ubuntu 22.04, Python 3.10 from the ROS installation, Node 22, Rust and protoc
35.1. A venv must expose the system ROS Python packages. In the repository root:

```bash
source /opt/ros/humble/setup.bash
python3 -m venv --system-site-packages .venv
source .venv/bin/activate
pip install 'maturin>=1.7,<2' 'protobuf>=7.35,<8'
python3 scripts/generate_python_msgs.py
python3 scripts/generate_rust_msgs.py
npm ci --prefix bindings/typescript
python3 scripts/generate_typescript_msgs.py
npm run build:ts --prefix bindings/typescript
(cd bindings/python && maturin develop --no-default-features --features extension-module,ws,console-api)
cd examples/ros2_web
npm ci
npm run build
export ROS_LOCALHOST_ONLY=1 ROS_DOMAIN_ID=73 RMW_IMPLEMENTATION=rmw_fastrtps_cpp
python3 run.py
# Or, after stopping run.py:
npx playwright install --with-deps chromium
bash test.sh
```

Install `ros-humble-example-interfaces` if missing. Custom demo ports are available
with `run.py --web-port 8081 --broker-port 15561`; open the printed URL, whose
`brokerPort` query configures the browser. The automated runner uses the default
ports and owns its backend lifecycle. Connection failures are shown in the UI;
after a disconnection, click Connect to rerun route readiness checks.

## Scope

This acceptance covers one ROS peer and one Python bridge. It does not establish
Jazzy, Rust/C++, Android, ROS-initiated service/action, multi-host DDS, restart
recovery, performance or long-running stability guarantees. See [support
status](support.md) for the wider matrix. No robot hardware is controlled.
