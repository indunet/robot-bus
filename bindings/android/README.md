# Android binding

Maven: `org.indunet:robot-bus-android`  
Package: `org.indunet.robot.bus` (same API as the JVM Kotlin binding)

Android Library (AAR) that:

1. Depends on [`../kotlin/`](../kotlin/) for the shared JNA API
2. Packages per-ABI `librobot_bus_c.so` under `src/main/jniLibs/`
3. Exposes `RobotBusAndroid.init(context)` to load the native library

## Local build

Requires Android SDK + NDK 26, `cmake`, `cargo-ndk`, Rust Android targets.

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
just android-dev
# = ./scripts/build_android_native.sh
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

dependencies {
  implementation("org.indunet:robot-bus-android:0.0.6")
}
```

## Layout

| Path | Role |
|------|------|
| `src/main/kotlin/.../RobotBusAndroid.kt` | `System.loadLibrary("robot_bus_c")` |
| `src/main/jniLibs/<abi>/` | Built by `scripts/build_android_native.sh` (gitignored) |
| `../kotlin/` | Shared JVM API module (included as `:robot-bus`) |

## Maven Central

Published by `.github/workflows/publish-maven-android.yml` (currently disabled
until GPG secrets are configured). JVM JAR uses a separate workflow:
`publish-maven-kotlin.yml`.
