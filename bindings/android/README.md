# Android binding

Maven: `org.indunet:robot-bus-android`  
Package: `org.indunet.robot.bus`  
**minSdk 24**, JVM target 11 · **standalone Kotlin SDK** (no dependency on `org.indunet:robot-bus`)

Android Library (AAR) that:

1. Ships a full Kotlin API (`Node`, `Broker`, parameters, typed protobuf, …) over JNA + C ABI
2. Embeds generated protobuf stubs under `generated/` (via `just gen-android`)
3. Packages per-ABI `librobot_bus_c.so` under `src/main/jniLibs/`
4. Exposes `RobotBusAndroid.init(context)` to `System.loadLibrary("robot_bus_c")`

Docs: [`docs/en/android-api.md`](../../docs/en/android-api.md). JVM Java binding (separate artifact): [`../java/`](../java/).

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

// implementation("org.indunet:robot-bus-android:1.3.4")
```

## Layout

| Path | Role |
|------|------|
| `src/main/kotlin/.../` | Kotlin SDK (Node, Broker, JNA, …) |
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
`published` or `workflow_dispatch`). Does **not** require installing the Java JAR.
