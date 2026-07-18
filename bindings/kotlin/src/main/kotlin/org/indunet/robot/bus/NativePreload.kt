package org.indunet.robot.bus

/**
 * Tell the JNA loader that `librobot_bus_c` is already in the process
 * (e.g. after Android `System.loadLibrary`). Prefer `RobotBusAndroid.init` on Android.
 */
fun markRobotBusNativePreloaded() {
    NativeLoader.markPreloaded()
}
