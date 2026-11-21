package org.indunet.robot.bus

import android.content.Context

/**
 * Android entry: load `librobot_bus_c.so` from the AAR / APK `jniLibs` before any API use.
 *
 * Call once from `Application.onCreate` (or before first [Node] / [Broker] use):
 *
 * ```kotlin
 * class App : Application() {
 *   override fun onCreate() {
 *     super.onCreate()
 *     RobotBusAndroid.init(this)
 *   }
 * }
 * ```
 *
 * The Node / Broker API itself is the shared Java SDK, compiled into this AAR.
 */
object RobotBusAndroid {
    @Volatile private var initialized = false

    @JvmStatic
    @Synchronized
    fun init(context: Context?) {
        if (!initialized) {
            System.loadLibrary("robot_bus_c")
            NativePreload.markRobotBusNativePreloaded()
            initialized = true
        }
        if (context != null) {
            AndroidRuntime.setIpcRoot(context.cacheDir.absolutePath)
        }
    }
}
