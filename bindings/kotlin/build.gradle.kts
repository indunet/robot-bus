plugins {
    kotlin("jvm") version "2.1.10"
    id("com.vanniktech.maven.publish") version "0.30.0"
}

group = providers.gradleProperty("GROUP").get()
version = providers.gradleProperty("VERSION_NAME").get()

repositories {
    mavenCentral()
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    api("net.java.dev.jna:jna:5.16.0")
    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
    // Native lib from `just cpp-dev` / cargo build of bindings/cpp/native
    environment(
        "ROBOT_BUS_NATIVE_DIR",
        rootProject.projectDir.resolve("../cpp/native/target/release").absolutePath,
    )
}

mavenPublishing {
    publishToMavenCentral(com.vanniktech.maven.publish.SonatypeHost.CENTRAL_PORTAL)
    signAllPublications()
    coordinates(
        providers.gradleProperty("GROUP").get(),
        providers.gradleProperty("POM_ARTIFACT_ID").get(),
        providers.gradleProperty("VERSION_NAME").get(),
    )
    pom {
        name.set(providers.gradleProperty("POM_NAME"))
        description.set(providers.gradleProperty("POM_DESCRIPTION"))
        inceptionYear.set(providers.gradleProperty("POM_INCEPTION_YEAR"))
        url.set(providers.gradleProperty("POM_URL"))
        licenses {
            license {
                name.set(providers.gradleProperty("POM_LICENSE_NAME"))
                url.set(providers.gradleProperty("POM_LICENSE_URL"))
                distribution.set(providers.gradleProperty("POM_LICENSE_DIST"))
            }
        }
        developers {
            developer {
                id.set(providers.gradleProperty("POM_DEVELOPER_ID"))
                name.set(providers.gradleProperty("POM_DEVELOPER_NAME"))
                url.set(providers.gradleProperty("POM_DEVELOPER_URL"))
            }
        }
        scm {
            url.set(providers.gradleProperty("POM_SCM_URL"))
            connection.set(providers.gradleProperty("POM_SCM_CONNECTION"))
            developerConnection.set(providers.gradleProperty("POM_SCM_DEV_CONNECTION"))
        }
    }
}
