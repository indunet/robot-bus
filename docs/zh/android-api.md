[English](../en/android-api.md) | 中文

# Android API

Maven: `org.indunet:robot-bus-android`  
Package: `org.indunet.robot.bus`  
**minSdk 24** · **独立 Kotlin SDK**（自有 Node / Broker / JNA + protobuf stubs + jniLibs）

不依赖 `org.indunet:robot-bus`（JVM Java 包）。两套产物各自完整、互不引用。

```bash
# Maven Central (when published)
# implementation("org.indunet:robot-bus-android:2.1.0")

# 本地：
just gen-android    # → bindings/android/generated/
just android-dev    # NDK .so + assembleRelease
just test-android   # host 单测（桌面 librobot_bus_c）
```

| 产物 | 坐标 | 说明 |
|------|------|------|
| Android AAR | `org.indunet:robot-bus-android` | Kotlin API + `jniLibs` + embedded protobuf stubs |

## 初始化

在 `Application.onCreate`（或首次使用 `Node` / `Broker` 之前）调用一次：

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
  implementation("org.indunet:robot-bus-android:2.1.0")
}
```

## Message bus（Kotlin）

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
    // createWallTimer；qosDepth；waitForMessage / waitForService
    // listParameters() → ListParametersResult；listAllParameters()；undeclareParameter
    // getParameter(name) → Parameter；取值 getParameterValue(name)

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

### HTTP 发现（填地址，不选传输）

对已知 API 口请求 `GET /api/v1/discover`：

```kotlin
val node = Node.discover(
    "talker", "tcp", DiscoverOpts(apiUrl = "http://127.0.0.1:15570"))
// 可选：brokerId、timeoutSecs；BrokerOptions.noDiscovery / domainId 非 UDP
```

跨 broker（federation）：

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

## 本地参数（Kotlin）

```kotlin
node.declareParameter("max_speed", 1.5)
node.setParameter("max_speed", 2.0)
val v = node.getParameter("max_speed") as Double
node.loadParametersFromYamlStr("ros__parameters:\n  max_speed: 3.0\n")
```

标量：`Boolean` / `Long` / `Double` / `String`。

## 测试

| 测试 | 命令 |
|------|------|
| `RobotBusAndroid.init`（无 jniLibs 时失败） | `just test-android` |
| 参数 + typed pub-sub | 同上（host 加载桌面 `librobot_bus_c`） |

## 本地开发

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
just gen-android
just android-dev       # 需 NDK 26 + cargo-ndk
just test-android
```

目录：[`bindings/android/`](../../bindings/android/)。JVM Java 绑定见 [`java-api.md`](java-api.md)。
