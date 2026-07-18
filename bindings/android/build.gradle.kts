plugins {
    id("com.android.library")
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

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    // Install first: cd ../java && mvn -DskipTests install
    api("org.indunet:robot-bus:0.0.6") {
        exclude(group = "net.java.dev.jna", module = "jna")
    }
    api("net.java.dev.jna:jna:5.16.0@aar")
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
            "Android AAR for robot-bus (JNA + librobot_bus_c jniLibs; topics, services, actions)",
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
