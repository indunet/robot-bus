package org.indunet.robot.bus;

import com.sun.jna.Native;
import com.sun.jna.Platform;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/**
 * Resolves the shared C ABI library built from {@code bindings/cpp/native}
 * ({@code librobot_bus_c} / {@code robot_bus_c}).
 *
 * <p>Search order:
 *
 * <ol>
 *   <li>Already loaded via {@link #markPreloaded()} (Android {@code RobotBusAndroid.init})
 *   <li>{@code ROBOT_BUS_NATIVE} / {@code -Drobot.bus.native} — full path to the shared library
 *   <li>{@code ROBOT_BUS_NATIVE_DIR} / {@code -Drobot.bus.native.dir} — directory containing it
 *   <li>Sibling release build: {@code bindings/cpp/native/target/release}
 *   <li>System / APK library name {@code robot_bus_c}
 * </ol>
 */
final class NativeLoader {
    private static volatile boolean preloaded = false;

    private NativeLoader() {}

    static void markPreloaded() {
        preloaded = true;
    }

    static RobotBusC loadLibrary() {
        if (preloaded || isAndroid()) {
            return Native.load("robot_bus_c", RobotBusC.class);
        }
        String explicit = System.getenv("ROBOT_BUS_NATIVE");
        if (explicit == null || explicit.isBlank()) {
            explicit = System.getProperty("robot.bus.native");
        }
        if (explicit != null && !explicit.isBlank()) {
            return Native.load(explicit, RobotBusC.class);
        }
        String dir = System.getenv("ROBOT_BUS_NATIVE_DIR");
        if (dir == null || dir.isBlank()) {
            dir = System.getProperty("robot.bus.native.dir");
        }
        List<Path> candidates = new ArrayList<>();
        if (dir != null && !dir.isBlank()) {
            candidates.add(Path.of(dir));
        }
        candidates.add(Path.of("").toAbsolutePath().resolve("../cpp/native/target/release").normalize());
        candidates.add(Path.of(System.getProperty("user.dir"), "bindings/cpp/native/target/release"));
        candidates.add(Path.of(System.getProperty("user.dir"), "../cpp/native/target/release").normalize());
        for (Path candidate : candidates) {
            Path file = findInDir(candidate);
            if (file != null) {
                return Native.load(file.toAbsolutePath().toString(), RobotBusC.class);
            }
        }
        return Native.load("robot_bus_c", RobotBusC.class);
    }

    private static boolean isAndroid() {
        try {
            Class.forName("android.os.Build");
            return true;
        } catch (ClassNotFoundException e) {
            return false;
        }
    }

    private static Path findInDir(Path dir) {
        if (!Files.isDirectory(dir)) {
            return null;
        }
        List<String> names;
        if (Platform.isMac()) {
            names = Arrays.asList("librobot_bus_c.dylib", "librobot_bus.dylib");
        } else if (Platform.isWindows()) {
            names = Arrays.asList("robot_bus_c.dll", "robot_bus.dll");
        } else {
            names = Arrays.asList("librobot_bus_c.so", "librobot_bus.so");
        }
        for (String name : names) {
            Path file = dir.resolve(name);
            if (Files.isRegularFile(file)) {
                return file;
            }
        }
        return null;
    }
}
