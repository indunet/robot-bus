package org.indunet.robot.bus;

import com.sun.jna.Native;
import com.sun.jna.Platform;
import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

/**
 * Resolves the shared C ABI library built from {@code bindings/cpp/native}
 * ({@code librobot_bus_c} / {@code robot_bus_c}).
 *
 * <p>Search order:
 *
 * <ol>
 *   <li>Already loaded via {@link #markPreloaded()} ({@code RobotBusAndroid.init})
 *   <li>{@code ROBOT_BUS_NATIVE} / {@code -Drobot.bus.native} — full path to the shared library
 *   <li>{@code ROBOT_BUS_NATIVE_DIR} / {@code -Drobot.bus.native.dir} — directory containing it
 *   <li>Sibling release build: {@code bindings/cpp/native/target/release}
 *   <li>System / APK library name {@code robot_bus_c}
 * </ol>
 *
 * <p>Uses {@link File} rather than {@code java.nio.file}, so the same sources run on Android API 24.
 * Host unit tests still have {@code android.os.Build} on the classpath, so this loader does not
 * treat that class as proof that {@code librobot_bus_c} is already loaded.
 */
final class NativeLoader {
    private static volatile boolean preloaded = false;

    private NativeLoader() {}

    static void markPreloaded() {
        preloaded = true;
    }

    static RobotBusC loadLibrary() {
        if (preloaded) {
            return Native.load("robot_bus_c", RobotBusC.class);
        }
        String explicit = firstNonBlank(System.getenv("ROBOT_BUS_NATIVE"), System.getProperty("robot.bus.native"));
        if (explicit != null) {
            return Native.load(explicit, RobotBusC.class);
        }
        String dir = firstNonBlank(
                System.getenv("ROBOT_BUS_NATIVE_DIR"), System.getProperty("robot.bus.native.dir"));
        List<File> candidates = new ArrayList<File>();
        if (dir != null) {
            candidates.add(new File(dir));
        }
        File cwd = new File("").getAbsoluteFile();
        candidates.add(new File(cwd, "../cpp/native/target/release"));
        String userDir = System.getProperty("user.dir");
        candidates.add(new File(userDir, "bindings/cpp/native/target/release"));
        candidates.add(new File(userDir, "../cpp/native/target/release"));
        for (File candidate : candidates) {
            File file = findInDir(normalize(candidate));
            if (file != null) {
                return Native.load(file.getAbsolutePath(), RobotBusC.class);
            }
        }
        return Native.load("robot_bus_c", RobotBusC.class);
    }

    private static String firstNonBlank(String first, String second) {
        if (!isBlank(first)) {
            return first;
        }
        if (!isBlank(second)) {
            return second;
        }
        return null;
    }

    private static boolean isBlank(String value) {
        return value == null || value.trim().isEmpty();
    }

    private static File normalize(File file) {
        try {
            return file.getCanonicalFile();
        } catch (IOException e) {
            return file.getAbsoluteFile();
        }
    }

    private static File findInDir(File dir) {
        if (dir == null || !dir.isDirectory()) {
            return null;
        }
        String[] names;
        if (Platform.isMac()) {
            names = new String[] {"librobot_bus_c.dylib", "librobot_bus.dylib"};
        } else if (Platform.isWindows()) {
            names = new String[] {"robot_bus_c.dll", "robot_bus.dll"};
        } else {
            names = new String[] {"librobot_bus_c.so", "librobot_bus.so"};
        }
        for (String name : names) {
            File file = new File(dir, name);
            if (file.isFile()) {
                return file;
            }
        }
        return null;
    }
}
