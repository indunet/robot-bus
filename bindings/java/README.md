# Java binding (JVM) — Maven

Maven: `org.indunet:robot-bus`  
Package: `org.indunet.robot.bus`  
**Bytecode target: Java 11+**

JNA wrappers over the C ABI (`bindings/cpp/native` → `librobot_bus_c`).

Android AAR lives next door: [`../android/`](../android/) (still Gradle/AGP; depends on this artifact via `mavenLocal()`).

## Local build

```bash
just java-dev
# cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
# cd bindings/java && mvn test
```

Install into `~/.m2` (needed before building the Android AAR locally):

```bash
cd bindings/java && mvn -DskipTests install
```

## Publish to Maven Central

Requires Central user token in `~/.m2/settings.xml` (`<server><id>central</id>…`) and GPG.

```bash
cd bindings/java
mvn -Prelease clean deploy
```

CI: `.github/workflows/publish-maven-java.yml` (currently disabled until GPG secrets are set).
