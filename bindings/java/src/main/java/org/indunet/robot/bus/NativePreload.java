package org.indunet.robot.bus;

/**
 * Tell the JNA loader that {@code librobot_bus_c} is already in the process (e.g. after Android
 * {@code System.loadLibrary}). Prefer {@code RobotBusAndroid.init} on Android.
 */
public final class NativePreload {
    private NativePreload() {}

    public static void markRobotBusNativePreloaded() {
        NativeLoader.markPreloaded();
    }
}
