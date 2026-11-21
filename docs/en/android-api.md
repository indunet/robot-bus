English | [中文](../zh/android-api.md)

# Android API

Maven: `org.indunet:robot-bus-android`  
Package: `org.indunet.robot.bus`  
**minSdk 24** · same implementation as the Java SDK, plus Android init (`RobotBusAndroid.init`) and `jniLibs`

The AAR compiles the sources in [`bindings/java`](../../bindings/java/src/main/java/org/indunet/robot/bus). It does not depend on the `org.indunet:robot-bus` JAR. Do not depend on the JAR and the AAR in the same app; the classes would be duplicated.

```bash
# Maven Central (when published)
# implementation("org.indunet:robot-bus-android:2.4.3")

# Local:
just gen-android    # → bindings/android/generated/
just android-dev    # NDK .so + assembleRelease
just test-android   # host unit tests (desktop librobot_bus_c)
```

| Artifact | Coordinates | Description |
|------|------|------|
| Android AAR | `org.indunet:robot-bus-android` | Shared Java API + `jniLibs` + embedded protobuf stubs |

## Initialization

Call once in `Application.onCreate` (or before the first use of `Node` / `Broker`):

```kotlin
class App : Application() {
  override fun onCreate() {
    super.onCreate()
    RobotBusAndroid.init(this) // System.loadLibrary("robot_bus_c")
  }
}
```

```kotlin
// build.gradle.kts
dependencies {
  implementation("org.indunet:robot-bus-android:2.4.3")
}
```

## Message bus

After `RobotBusAndroid.init(this)` receives an Android `Context`, the embedded broker creates a temporary IPC directory under the app's private `cacheDir` and removes it on shutdown. Other processes of the same app can obtain the actual path with `Node.discover(name, "ipc", opts)`. For `inproc`, the broker and nodes must still share the same Robot Bus `Context`. Set `brokerOptions(tcpOnly = true)` to disable both local transports.

`Node`, `Broker`, and the other types are the Java SDK. Kotlin cannot pass named arguments to those methods (for example `qosDepth = 5`); pass them by position, or use an overload. Build option objects with `brokerOptions`, `nodeOptions`, and `discoverOpts`, which keep default arguments.

There is no standalone CLI on Android; **starting the broker in-process** (`Broker()`) is the recommended usage:

```kotlin
import org.indunet.robot.bus.Broker
import org.indunet.robot.bus.Node
import org.indunet.robot.bus.discoverOpts
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu

Broker().use { _ ->
  Node.discover("pilot", "tcp", discoverOpts(apiUrl = "http://127.0.0.1:15560")).use { node ->
    val imuPub = node.createPublisher("/robot1/imu", Imu::class.java)
    val sub = node.createSubscription(
      "/robot1/imu",
      { imu -> println("z=${imu.angularVelocity.z}") },
      Imu::class.java,
    )
    // sub.destroy()
    // createWallTimer; qosDepth; waitForMessage / waitForService
    // listParameters() → ListParametersResult; listAllParameters(); undeclareParameter
    // getParameter(name) → Parameter; getParameterValue(name)

    imuPub.publish(
      Imu.newBuilder()
        .setAngularVelocity(Vector3.newBuilder().setZ(0.1).build())
        .setLinearAcceleration(Vector3.newBuilder().setZ(9.8).build())
        .build(),
    )
    // node.spin()
  }
}
```

### HTTP discovery (fill addresses, pick transport yourself)

Request `GET /api/v1/discover` on a known API base URL:

```kotlin
val node = Node.discover(
    "talker", "tcp", discoverOpts(apiUrl = "http://127.0.0.1:15560"))
// Optional: brokerId, timeoutSecs. BrokerOptions noDiscovery / domainId are not UDP.
```

Cross-broker (federation):

```kotlin
import org.indunet.robot.bus.Broker
import org.indunet.robot.bus.brokerOptions

Broker(
  brokerOptions(
    brokerId = "broker-a",
    messagePeers = listOf("tcp://10.0.0.2:15581"),
    servicePeers = listOf("broker-b=tcp://10.0.0.2:15663"),
    actionPeers = listOf("broker-b=tcp://10.0.0.2:15665"),
    tcpOnly = true,
    noConsole = true,
  ),
).use { _ ->
  // …
}
```

## Local parameters

```kotlin
node.declareParameter("max_speed", 1.5)
node.setParameter("max_speed", 2.0)
val v = node.getParameterValue("max_speed") as Double
node.loadParametersFromYamlStr("ros__parameters:\n  max_speed: 3.0\n")
```

Scalars: `Boolean` / `Long` / `Double` / `String`.

## Testing

| Test | Command |
|------|------|
| `RobotBusAndroid.init` (fails without jniLibs) | `just test-android` |
| Parameters + typed pub-sub | Same (host loads desktop `librobot_bus_c`) |

## Local development

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
just gen-android
just android-dev       # requires NDK 26 + cargo-ndk
just test-android
```

Directory: [`bindings/android/`](../../bindings/android/). For JVM Java bindings, see [`java-api.md`](java-api.md).
