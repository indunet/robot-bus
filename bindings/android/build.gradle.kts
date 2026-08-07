plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("com.vanniktech.maven.publish")
}

group = providers.gradleProperty("GROUP").get()
version = providers.gradleProperty("VERSION_NAME").get()

android {
    namespace = "org.indunet.robot.bus"
    compileSdk = 35

    defaultConfig {
        minSdk = 24
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    kotlinOptions {
        jvmTarget = "11"
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
            java.srcDir("generated")
        }
    }

    testOptions {
        unitTests.isReturnDefaultValues = true
        unitTests.all {
            it.environment(
                "ROBOT_BUS_NATIVE_DIR",
                System.getenv("ROBOT_BUS_NATIVE_DIR")
                    ?: "${project.projectDir}/../cpp/native/target/release",
            )
        }
    }
}

dependencies {
    // Device: JNA Android AAR. Host unit tests also need the platform JAR (jnidispatch).
    api("net.java.dev.jna:jna:5.16.0@aar")
    api("com.google.protobuf:protobuf-java:4.35.1")

    testImplementation("net.java.dev.jna:jna:5.16.0")
    testImplementation("junit:junit:4.13.2")
}

// Both variants start native brokers on localhost; do not let their test JVMs compete for ports.
tasks.matching { it.name == "testReleaseUnitTest" }.configureEach {
    mustRunAfter("testDebugUnitTest")
}

mavenPublishing {
    configure(com.vanniktech.maven.publish.AndroidSingleVariantLibrary())
    publishToMavenCentral(com.vanniktech.maven.publish.SonatypeHost.CENTRAL_PORTAL)
    signAllPublications()
    coordinates(
        providers.gradleProperty("GROUP").get(),
        "robot-bus-android",
        providers.gradleProperty("VERSION_NAME").get(),
    )
    pom {
        name.set("robot-bus-android")
        description.set(
            "Standalone Android Kotlin SDK for robot-bus (JNA + librobot_bus_c jniLibs)",
        )
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
                email.set(providers.gradleProperty("POM_DEVELOPER_EMAIL"))
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
