# Kotlin binding (JVM)

Maven: `org.indunet:robot-bus`  
Package: `org.indunet.robot.bus`

JNA wrappers over the C ABI (`bindings/cpp/native` → `librobot_bus_c`).

Android AAR lives next door: [`../android/`](../android/).

## Local build

```bash
just kotlin-dev
# cargo build --release --manifest-path bindings/cpp/native/Cargo.toml
# cd bindings/kotlin && ./gradlew test
```

## Maven Central

Published by `.github/workflows/publish-maven-kotlin.yml` (currently disabled
until GPG secrets are configured). Android AAR has a separate workflow:
`publish-maven-android.yml`.
