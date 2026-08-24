English | [中文](../zh/android-api.md)

# Android API

Maven: `org.indunet:robot-bus-android`  
Package: `org.indunet.robot.bus`  
**minSdk 24** · **Independent Kotlin SDK** (own Node / Broker / JNA + protobuf stubs + jniLibs)

Does not depend on `org.indunet:robot-bus` (JVM Java package). Both artifacts are complete and independent of each other.

```bash
# Maven Central (when published)
# implementation("org.indunet:robot-bus-android:1.3.2")

# Local:
just gen-android    # → bindings/android/generated/
just android-dev    # NDK .so + assembleRelease
just test-android   # host unit tests (desktop librobot_bus_c)
```

| Artifact | Coordinates | Description |
|------|------|------|
| Android AAR | `org.indunet:robot-bus-android` | Kotlin API + `jniLibs` + embedded protobuf stubs |

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
  implementation("org.indunet:robot-bus-android:1.3.2")
}
```

## Message bus (Kotlin)

```kotlin
import org.indunet.robot.bus.Broker
import org.indunet.robot.bus.Node
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu

Broker().use { _ ->
  Node("pilot").use { node ->
    val imuPub = node.createPublisher("/robot1/imu", Imu::class.java)
    val sub = node.createSubscription(
      "/robot1/imu",
      { topic, imu -> println("$topic z=${imu.angularVelocity.z}") },
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
    "talker", "tcp", DiscoverOpts(apiUrl = "http://127.0.0.1:15570"))
// Optional: brokerId, timeoutSecs; BrokerOptions.noDiscovery / domainId are not UDP
```

Cross-broker (federation):

```kotlin
Broker(
  BrokerOptions(
    brokerId = "broker-a",
    messagePeers = listOf("tcp://10.0.0.2:15561"),
    servicePeers = listOf("broker-b=tcp://10.0.0.2:15663"),
    actionPeers = listOf("broker-b=tcp://10.0.0.2:15665"),
    tcpOnly = true,
    noConsole = true,
  ),
).use { _ ->
  // …
}
```

## Local parameters (Kotlin)

```kotlin
node.declareParameter("max_speed", 1.5)
node.setParameter("max_speed", 2.0)
val v = node.getParameter("max_speed") as Double
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
