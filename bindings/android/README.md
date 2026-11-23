# Android binding

Maven: `org.indunet:robot-bus-android`  
Package: `org.indunet.robot.bus`  
**minSdk 24**, JVM target 11

Android Library (AAR) that:

1. Compiles the shared Java SDK from [`../java/src/main/java`](../java/src/main/java) (Node, Broker, JNA, typed protobuf)
2. Embeds generated protobuf stubs under `generated/` (via `just gen-android`)
3. Packages per-ABI `librobot_bus_c.so` under `src/main/jniLibs/`
4. Adds `RobotBusAndroid.init(context)`, which loads that library and points the broker IPC root at the app cache directory
5. Adds Kotlin helpers `brokerOptions`, `nodeOptions`, and `discoverOpts` for default arguments

It does not depend on the `org.indunet:robot-bus` JAR. Do not put both artifacts on one app classpath.

Docs: [`docs/en/android-api.md`](../../docs/en/android-api.md). JVM JAR: [`../java/`](../java/).

## Local build

Requires Android SDK + NDK 26, `cmake`, `cargo-ndk`, Rust Android targets, and `protoc` 35.1.

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
just android-dev
# = gen-android
# + ./scripts/build_android_native.sh
# + cd bindings/android && ./gradlew assembleRelease
```

App usage:

```kotlin
class App : Application() {
  override fun onCreate() {
    super.onCreate()
    RobotBusAndroid.init(this)
  }
}

// implementation("org.indunet:robot-bus-android:3.0.0")
```

## Layout

| Path | Role |
|------|------|
| `src/main/kotlin/.../` | `RobotBusAndroid.init` and Kotlin option helpers |
| `bindings/java/src/main/java` | Shared SDK sources compiled into this AAR |
| `src/test/kotlin/.../` | Host unit tests |
| `generated/` | Protobuf Java stubs (`just gen-android`, gitignored) |
| `src/main/jniLibs/<abi>/` | Built by `scripts/build_android_native.sh` (gitignored) |

## Tests

```bash
just test-android
# builds desktop librobot_bus_c, then Gradle unit tests
```

## Maven Central

Published by `.github/workflows/publish-maven-android.yml` (GitHub Release
`published` or `workflow_dispatch`). The AAR compiles `bindings/java` sources
directly and does **not** require installing the Java JAR.
