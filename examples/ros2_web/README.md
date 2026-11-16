# ROS2 browser bridge lab

[中文说明](../../docs/zh/ros2-web.md) | [English guide](../../docs/en/ros2-web.md)

A browser using the **local TypeScript SDK** connects through WebSocket, an
embedded Robot Bus broker and the **Python/rclpy bridge** to a separate, real ROS2
process. No ROS or WebSocket mocks are used. The first acceptance target is
**Humble + Fast DDS + Chromium**; this does not certify other bridges or platforms.

From the repository root, with Docker Compose available:

```bash
docker compose -f examples/ros2_web/compose.yaml up --build
```

Open <http://127.0.0.1:8080>. Ports 8080 and 15560 must be free. The first build
downloads ROS/build/browser dependencies and compiles this checkout's SDKs.
The browser sends protobuf bytes using the public SDK; JSON appears only inside
the demo's `std_msgs/String` status payload.

Stop with Ctrl+C, then `docker compose -f examples/ros2_web/compose.yaml down`.

For automated acceptance (stop the interactive demo first if running locally):

```bash
docker build -f examples/ros2_web/Dockerfile -t robot-bus-ros2-web:test .
docker run --name ros2-web-acceptance --init \
  -e ROBOT_BUS_TEST_COMMIT="$(git rev-parse HEAD)" \
  robot-bus-ros2-web:test bash test.sh
docker cp ros2-web-acceptance:/workspace/examples/ros2_web/test-results ./ros2-web-results
docker rm ros2-web-acceptance
```

Run the copy command even if the test command fails. The container is retained
so failures can be inspected; choose a new name or remove it before rerunning.
See the guides for route semantics, local development, evidence and limitations.
