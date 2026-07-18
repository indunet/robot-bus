# Android binding

Maven: `org.indunet:robot-bus-android`  
Package: `org.indunet.robot.bus` (same Java API as the JVM binding)  
**minSdk 24**, Java 11 language level

Android Library (AAR) that:

1. Depends on [`../java/`](../java/) Maven artifact `org.indunet:robot-bus` (via `mavenLocal()` after `mvn install`) — **same API + protobuf stubs**
2. Packages per-ABI `librobot_bus_c.so` under `src/main/jniLibs/`
3. Exposes `RobotBusAndroid.init(context)` to load the native library

API examples: [`docs/java-api.md`](../../docs/java-api.md).

## Local build

Requires Android SDK + NDK 26, `cmake`, `cargo-ndk`, Rust Android targets, and a prior
`mvn install` of the Java SDK.

```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
just android-dev
# = mvn -DskipTests install (bindings/java)
# + ./scripts/build_android_native.sh
# + cd bindings/android && ./gradlew assembleRelease
```

App usage:

```java
public class App extends Application {
  @Override public void onCreate() {
    super.onCreate();
    RobotBusAndroid.init(this);
  }
}

// implementation("org.indunet:robot-bus-android:0.0.6")
```

## Layout

| Path | Role |
|------|------|
| `src/main/java/.../RobotBusAndroid.java` | `System.loadLibrary("robot_bus_c")` |
| `src/main/jniLibs/<abi>/` | Built by `scripts/build_android_native.sh` (gitignored) |
| `../java/` | Shared JVM API (`org.indunet:robot-bus` via mavenLocal) |

## Maven Central

Published by `.github/workflows/publish-maven-android.yml` (currently disabled
until GPG secrets are configured). JVM JAR uses a separate workflow:
`publish-maven-java.yml`.
