# Kotlin binding

JVM Kotlin SDK over the shared C ABI (`bindings/cpp/native` → `librobot_bus_c`).
Package: `org.indunet.robot.bus`. Maven coordinates: `org.indunet:robot-bus`.

## Layout

| Path | Role |
|------|------|
| `src/main/kotlin/org/indunet/robot/bus/` | Idiomatic Kotlin wrappers (JNA) |
| `build.gradle.kts` | JVM 17 + Maven Central publish (`com.vanniktech.maven.publish`) |
| `../cpp/native/` | Native library source (build with `just cpp-dev` / cargo) |

Protobuf message stubs for Kotlin are not generated yet; publish/subscribe
payloads are raw `ByteArray` (same model as the C++ binding before typed msgs).

## Local build

```bash
# 1) Build the shared library (librobot_bus_c)
just cpp-dev
# or: cargo build --release --manifest-path bindings/cpp/native/Cargo.toml

# 2) Compile / test Kotlin
just kotlin-dev
# or: cd bindings/kotlin && ./gradlew test
```

Native discovery (first match wins):

1. `ROBOT_BUS_NATIVE` — absolute path to the `.dylib` / `.so` / `.dll`
2. `ROBOT_BUS_NATIVE_DIR` — directory containing the library
3. `bindings/cpp/native/target/release/`
4. System library name `robot_bus_c`

## Maven Central

Publish workflow: `.github/workflows/publish-maven.yml` (currently **disabled**
until GPG signing secrets are configured). Required GitHub Actions secrets:

- `MAVEN_CENTRAL_USERNAME` / `MAVEN_CENTRAL_PASSWORD` — Central Portal user token
- `ORG_GRADLE_PROJECT_signingInMemoryKey` / `ORG_GRADLE_PROJECT_signingInMemoryKeyPassword`
  (or `GPG_PRIVATE_KEY` / `GPG_PASSPHRASE` wired in the workflow)

When enabled, a release tag `v*` matching `VERSION_NAME` in `gradle.properties`
will publish `org.indunet:robot-bus`.
