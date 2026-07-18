# Java binding (JVM) — Maven

Maven: `org.indunet:robot-bus`  
Package: `org.indunet.robot.bus`  
**Bytecode target: Java 11+**

JNA wrappers over the C ABI (`bindings/cpp/native` → `librobot_bus_c`), plus
**pre-generated protobuf stubs** (same `proto/` as Python / C++ / TypeScript).

Android AAR: [`../android/`](../android/) (depends on this artifact via `mavenLocal()` /
Maven Central).

API examples: [`docs/java-api.md`](../../docs/java-api.md).

## Local build

```bash
just java-dev
# = just gen-java
# + cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
# + cd bindings/java && mvn test
```

Needs **protoc 35.1** for `just gen-java` (stubs under `generated/`, gitignored).
Published JARs embed stubs — consumers do not need protoc.

Install into `~/.m2` (needed before building the Android AAR locally):

```bash
just java-install
```

## Typed protobuf

```java
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;

TypedTopicPublisher<Imu> pub = node.createPublisher("/imu", Imu.class);
node.createSubscription("/imu", (topic, imu) -> { /* … */ }, Imu.class);
```

Omit the `Class` argument to stay on raw `byte[]`.

## Publish to Maven Central

Requires Central user token in `~/.m2/settings.xml` (`<server><id>central</id>…`) and GPG.

```bash
just gen-java
cd bindings/java
mvn -Prelease clean deploy
```

CI: `.github/workflows/publish-maven-java.yml` (currently disabled until GPG secrets are set).
