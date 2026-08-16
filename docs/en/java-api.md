English | [中文](../zh/java-api.md)

# Java API

```bash
# Maven Central (when published)
# implementation("org.indunet:robot-bus:1.1.0")           // JVM

# Local:
just java-dev       # gen-java + cargo FFI + mvn test
```

| Artifact | Coordinates | Description |
|------|------|------|
| JVM JAR | `org.indunet:robot-bus` | Java 11+, JNA + protobuf stubs |

Package name `org.indunet.robot.bus`. The Android AAR is an **independent** Kotlin SDK (does not depend on this JAR); see [`android-api.md`](android-api.md).
## Broker startup

Same as other languages: start the broker first, then run your application logic.

```bash
robot-bus-broker
# or robot-bus-broker --grpc-listen 0.0.0.0:15570 --tcp-only
```

In-process:

```java
import org.indunet.robot.bus.Broker;

try (Broker broker = new Broker()) {
  // broker binds by default; business nodes connect to localhost
}
```

Cross-broker (federation) uses the same string conventions as the CLI:

```java
import java.util.List;
import org.indunet.robot.bus.Broker;
import org.indunet.robot.bus.BrokerOptions;

BrokerOptions opts =
    new BrokerOptions(
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        true,
        true,
        "broker-a",
        List.of("tcp://10.0.0.2:15561"),
        List.of("broker-b=tcp://10.0.0.2:15663"),
        List.of("broker-b=tcp://10.0.0.2:15665"));
try (Broker broker = new Broker(opts)) {
  // …
}
```

## Message bus (Node + spin)

Similar to ROS 2: `Node` → `createPublisher` / `createSubscription` → `spin()`.

The primary path is **typed** (pass a protobuf `Class` at creation time for automatic `toByteArray` / `parseFrom`); omitting the type still yields raw `byte[]`.

```java
import org.indunet.robot.bus.Broker;
import org.indunet.robot.bus.Node;
import org.indunet.robot.bus.TypedTopicPublisher;
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3;
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;

try (Broker broker = new Broker();
     Node node = new Node("pilot")) {
  TypedTopicPublisher<Imu> imuPub = node.createPublisher("/robot1/imu", Imu.class);
  var sub = node.createSubscription(
      "/robot1/imu",
      (topic, imu) -> System.out.println(topic + " z=" + imu.getAngularVelocity().getZ()),
      Imu.class);
  // sub.destroy();
  // createWallTimer; optional qosDepth; waitForMessage / waitForService
  // listParameters() → {names, prefixes}; listAllParameters(); undeclareParameter
  // getParameter(name) → Parameter; or getParameterValue(name)

  imuPub.publish(
      Imu.newBuilder()
          .setAngularVelocity(Vector3.newBuilder().setZ(0.1).build())
          .setLinearAcceleration(Vector3.newBuilder().setZ(9.8).build())
          .build());

  // node.spin();
}
```

Raw bytes (compatible with legacy usage):

```java
var pub = node.createPublisher("/robot1/imu");
pub.publish(imu.toByteArray());

node.createSubscription("/robot1/imu", (topic, payload) -> {
  Imu msg = Imu.parseFrom(payload);
});
```

### Android

See [`android-api.md`](android-api.md) (independent Kotlin SDK; `RobotBusAndroid.init`, Node, parameters, etc.).

### HTTP discovery (fill addresses, pick transport yourself)

Request `GET /api/v1/discover` on a known API base URL to obtain connectable ZMQ endpoints. You still choose the transport; discovery only fills in locations:

```java
Node node = Node.discover(
    "talker", "tcp", new DiscoverOpts("http://127.0.0.1:15570"));
// Optional: brokerId, timeoutSecs; null / 0 → defaults
// BrokerOptions.noDiscovery / domainId are soft labels for compatibility, not UDP multicast
```

### WebSocket RPC mode Node (client)
`Node.ws` / `Node.wsAt` connect through the broker WebSocket RPC gateway and do not create ZMQ sockets.

| Supported | Not supported |
|------|--------|
| `createSubscription` | `createService` |
| `createPublisher` | `createActionServer` |
| `createClient` | — |
| `createActionClient` | |
| `createTimer`, `spin` / `shutdown` | |

```java
Node node = Node.ws("web-client");
// or Node.wsAt("web-client", "http://127.0.0.1:15570");
```

### Service / Action (typed)

```java
import org.indunet.robot.bus.TypedActionPhase;
import org.indunet.robot.bus.example_interfaces.action.v1.FibonacciFeedback;
import org.indunet.robot.bus.example_interfaces.action.v1.FibonacciGoal;
import org.indunet.robot.bus.example_interfaces.action.v1.FibonacciResult;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolRequest;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolResponse;
import java.util.List;

node.createService(
    "/set_bool",
    req -> SetBoolResponse.newBuilder()
        .setSuccess(true)
        .setMessage("set:" + req.getData())
        .build(),
    SetBoolRequest.class,
    SetBoolResponse.class);

var svc = node.createClient("/set_bool", SetBoolRequest.class, SetBoolResponse.class);
// SetBoolResponse reply = svc.call(SetBoolRequest.newBuilder().setData(true).build(), 5.0);

node.createActionServer(
    "/fibonacci",
    goal -> {
      // return FEEDBACK / RESULT etc. phases
      return List.of(
          new TypedActionPhase(
              "RESULT",
              FibonacciResult.newBuilder().addSequence(goal.getOrder()).build()));
    },
    FibonacciGoal.class,
    FibonacciFeedback.class,
    FibonacciResult.class);

var act =
    node.createActionClient(
        "/fibonacci", FibonacciGoal.class, FibonacciFeedback.class, FibonacciResult.class);
// List<TypedActionMessage> events = act.sendGoal(FibonacciGoal.newBuilder().setOrder(5).build(), null, 10.0);
```

### Executor / callback group

```java
MultiThreadedExecutor executor = new MultiThreadedExecutor(4);
executor.addNode(node);
CallbackGroup group = node.createCallbackGroup(CallbackGroupType.Reentrant);
node.createSubscription("/robot1/imu", cb, Imu.class, group);
```

## Working with Protobuf

Message packages live under `org.indunet.robot.bus.<pkg>.{msg|srv|action}.v1` (aligned with Python `robot_bus.<pkg>…` / C++ `robot_bus/…`):

| Language | Path |
|------|------|
| Java / Android | `import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;` (Android see [android-api.md](android-api.md)) |
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |
| C++ | `#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>` |

After local proto changes:

```bash
just gen-java   # → bindings/java/generated/ (gitignored; protoc 35.1)
just java-dev
```

JAR/AAR published to Maven Central **embed** stubs; consumers do not need `protoc`. Runtime dependency: `com.google.protobuf:protobuf-java` (aligned with protoc 35.x, currently pinned `4.35.1`).

## Local development

```bash
just gen-java
just java-dev          # includes MsgsRoundtripTest + EndpointSmokeTest
just java-install      # ~/.m2 (JVM only; Android no longer needed)
```

## Current Java API overview

| Symbol | Description |
|------|------|
| `Node(name)` / `Node(name, NodeOptions)` | Create a node |
| `Node.tcp` / `ipc` / `inproc` / `inproc(Context, …)` / `withContext` / `ws` / `wsAt` / `discover` | Transport presets; same-process inproc must share `Context`; `discover` only fills in the address |
| `declareParameter` / `getParameter` / `setParameter` / `hasParameter` / `listParameters` | Local parameters on this node (Boolean / Long / Double / String) |
| `loadParametersFromYaml` / `loadParametersFromYamlStr` | Load parameters from a YAML file or string |
| `Parameter` | name/value from `getParameter` / `listAllParameters` |
| `ListParametersResult` | `{names, prefixes}` from `listParameters` |
| `SubscriptionHandle` / `ServiceHandle` / `ActionServerHandle` | returned by `create*`; `destroy()` / `close()` |
| `node.spin()` / `spinOnce` / `shutdown` | Drive callbacks |
| `createPublisher(topic)` / `createPublisher(topic, Class<T>)` | raw → `TopicPublisher`; typed → `TypedTopicPublisher` |
| `createSubscription(..., MsgCallback)` / `(..., TypedMsgCallback, Class)` | raw `byte[]` or typed `Message` |
| `createService` / `createClient` | raw or typed (`Request`/`Response` Class) |
| `createActionServer` / `createActionClient` | raw or typed (goal/feedback/result Class) |
| `Context` | Shared ZMQ context (required for same-process inproc) |
| `Broker` / `Broker(Context, …)` | In-process broker |
| `SingleThreadedExecutor` / `MultiThreadedExecutor` (can pass `Context`) | Explicit executor |

WebSocket RPC mode is covered in the previous section; the underlying C ABI is shared with C++ via `librobot_bus_c`. For the Android entry point, see [`android-api.md`](android-api.md).
