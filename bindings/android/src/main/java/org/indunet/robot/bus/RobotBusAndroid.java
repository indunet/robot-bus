package org.indunet.robot.bus;

import android.content.Context;

/**
 * Android entry: load {@code librobot_bus_c.so} from the AAR / APK {@code jniLibs} before any API
 * use.
 *
 * <p>Call once from {@code Application.onCreate} (or before first {@link Node} / {@link Broker}
 * use):
 *
 * <pre>{@code
 * class App extends Application {
 *   @Override
 *   public void onCreate() {
 *     super.onCreate();
 *     RobotBusAndroid.init(this);
 *   }
 * }
 * }</pre>
 */
public final class RobotBusAndroid {
    private static boolean initialized = false;

    private RobotBusAndroid() {}

    public static synchronized void init(Context context) {
        if (initialized) {
            return;
        }
        System.loadLibrary("robot_bus_c");
        NativePreload.markRobotBusNativePreloaded();
        initialized = true;
    }
}
