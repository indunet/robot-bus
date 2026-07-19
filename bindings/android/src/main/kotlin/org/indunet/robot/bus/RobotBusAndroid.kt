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
 */
object RobotBusAndroid {
    @Volatile private var initialized = false

    @JvmStatic
    @Synchronized
    fun init(context: Context?) {
        if (initialized) {
            return
        }
        System.loadLibrary("robot_bus_c")
        NativePreload.markRobotBusNativePreloaded()
        initialized = true
    }
}
