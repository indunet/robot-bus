# Java / Android API

```bash
# Maven Central (when published)
# implementation("org.indunet:robot-bus:0.0.6")           // JVM
# implementation("org.indunet:robot-bus-android:0.0.6")   // Android AAR

# 本地：
just java-dev       # gen-java + cargo FFI + mvn test
just android-dev    # java-install + NDK .so + assembleRelease
```

| 产物 | 坐标 | 说明 |
|------|------|------|
| JVM JAR | `org.indunet:robot-bus` | Java 11+，JNA + protobuf stubs |
| Android AAR | `org.indunet:robot-bus-android` | 依赖上表 JAR + `jniLibs`；`RobotBusAndroid.init` |

包名均为 `org.indunet.robot.bus`。Android 与 JVM **同一套 Java API**（含 typed protobuf）；AAR 额外打进各 ABI 的 `librobot_bus_c.so`。

## Broker 启动

与其他语言相同：先起 broker，再跑业务。

```bash
robot-bus-broker
# 或 robot-bus-broker --grpc-listen 0.0.0.0:15770 --tcp-only
```

进程内：

```java
import org.indunet.robot.bus.Broker;

try (Broker broker = new Broker()) {
  // broker 默认 bind；业务节点连本机即可
}
```

## Message bus（Node + spin）

接近 ROS 2：`Node` → `createPublisher` / `createSubscription` → `spin()`。

主推 **typed**（创建时传入 protobuf `Class`，自动 `toByteArray` / `parseFrom`）；不传类型则仍为 raw `byte[]`。

```java
import org.indunet.robot.bus.Broker;
import org.indunet.robot.bus.Node;
import org.indunet.robot.bus.TypedTopicPublisher;
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3;
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;

try (Broker broker = new Broker();
     Node node = new Node("pilot")) {
  TypedTopicPublisher<Imu> imuPub = node.createPublisher("/robot1/imu", Imu.class);
  node.createSubscription(
      "/robot1/imu",
      (topic, imu) -> System.out.println(topic + " z=" + imu.getAngularVelocity().getZ()),
      Imu.class);

  imuPub.publish(
      Imu.newBuilder()
          .setAngularVelocity(Vector3.newBuilder().setZ(0.1).build())
          .setLinearAcceleration(Vector3.newBuilder().setZ(9.8).build())
          .build());

  // node.spin();
}
```

Raw bytes（兼容旧用法）：

```java
var pub = node.createPublisher("/robot1/imu");
pub.publish(imu.toByteArray());

node.createSubscription("/robot1/imu", (topic, payload) -> {
  Imu msg = Imu.parseFrom(payload);
});
```

### Android

```java
public class App extends Application {
  @Override
  public void onCreate() {
    super.onCreate();
    RobotBusAndroid.init(this); // System.loadLibrary("robot_bus_c")
  }
}
```

之后与 JVM 相同：`new Node(...)`、typed `createPublisher` 等。消息类型同样来自 `org.indunet.robot.bus.sensor_msgs.msg.v1.Imu`（经 `robot-bus` 传递依赖带入）。

### gRPC 模式 Node（客户端）

`Node.grpc` / `Node.grpcAt` 经 broker gRPC 网关接入，不创建 ZMQ socket。

| 支持 | 不支持 |
|------|--------|
| `createSubscription` | `createPublisher` |
| `createClient` | `createService` |
| `createActionClient` | `createActionServer` |
| `createTimer`、`spin` / `shutdown` | — |

```java
Node node = Node.grpc("web-client");
// 或 Node.grpcAt("web-client", "http://127.0.0.1:15770");
```

### Service / Action（typed）

```java
import org.indunet.robot.bus.TypedActionPhase;
import org.indunet.robot.bus.robot_bus_interface.action.v1.FibonacciFeedback;
import org.indunet.robot.bus.robot_bus_interface.action.v1.FibonacciGoal;
import org.indunet.robot.bus.robot_bus_interface.action.v1.FibonacciResult;
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
      // 返回 FEEDBACK / RESULT 等 phase
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

## 与 Protobuf 配合

消息包挂在 `org.indunet.robot.bus.<pkg>.{msg|srv|action}.v1`（与 Python `robot_bus.<pkg>…` / C++ `robot_bus/…` 对齐）：

| 语言 | 路径 |
|------|------|
| Java / Android | `import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;` |
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |
| C++ | `#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>` |

本地改 proto 后：

```bash
just gen-java   # → bindings/java/generated/（gitignored；protoc 35.1）
just java-dev
```

发布到 Maven Central 的 JAR/AAR **已嵌入** stubs；消费方不需要 `protoc`。运行时依赖 `com.google.protobuf:protobuf-java`（与 protoc 35.x 对齐，当前 pin `4.35.1`）。

## 本地开发

```bash
just gen-java
just java-dev          # 含 MsgsRoundtripTest + EndpointSmokeTest
just java-install      # ~/.m2，供 android-dev
just android-dev       # 需 ANDROID_HOME + NDK 26 + cargo-ndk
```

## 当前 Java API 一览

| 符号 | 说明 |
|------|------|
| `Node(name)` / `Node(name, NodeOptions)` | 建节点 |
| `Node.tcp` / `ipc` / `inproc` / `grpc` / `grpcAt` | 传输预设 |
| `node.spin()` / `spinOnce` / `shutdown` | 驱动回调 |
| `createPublisher(topic)` / `createPublisher(topic, Class<T>)` | raw → `TopicPublisher`；typed → `TypedTopicPublisher` |
| `createSubscription(..., MsgCallback)` / `(..., TypedMsgCallback, Class)` | raw `byte[]` 或 typed `Message` |
| `createService` / `createClient` | raw 或 typed（`Request`/`Response` Class） |
| `createActionServer` / `createActionClient` | raw 或 typed（goal/feedback/result Class） |
| `Broker` | 进程内 broker |
| `SingleThreadedExecutor` / `MultiThreadedExecutor` | 显式执行器 |
| `RobotBusAndroid.init(Context)` | Android 加载 native（仅 AAR） |

gRPC 模式见上一节；底层 C ABI 与 C++ 共用 `librobot_bus_c`。
