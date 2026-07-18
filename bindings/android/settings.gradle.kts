pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("org.jetbrains.kotlin.jvm") version "2.1.10"
        id("com.android.library") version "8.7.3"
        id("org.jetbrains.kotlin.android") version "2.1.10"
        id("com.vanniktech.maven.publish") version "0.30.0"
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.PREFER_SETTINGS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "robot-bus-android"

// JVM API module lives next door under bindings/kotlin
include(":robot-bus")
project(":robot-bus").projectDir = file("../kotlin")
