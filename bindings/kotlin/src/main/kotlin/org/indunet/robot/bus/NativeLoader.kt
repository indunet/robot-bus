package org.indunet.robot.bus

import com.sun.jna.Native
import com.sun.jna.Platform
import java.nio.file.Files
import java.nio.file.Path

/**
 * Resolves the shared C ABI library built from `bindings/cpp/native`
 * (`librobot_bus_c` / `robot_bus_c`).
 *
 * Search order:
 * 1. `ROBOT_BUS_NATIVE` / `-Drobot.bus.native` — full path to the shared library
 * 2. `ROBOT_BUS_NATIVE_DIR` / `-Drobot.bus.native.dir` — directory containing it
 * 3. Sibling release build: `bindings/cpp/native/target/release`
 * 4. System library name `robot_bus_c` (java.library.path / DYLD_LIBRARY_PATH)
 */
internal object NativeLoader {
    fun loadLibrary(): RobotBusC {
        val explicit = System.getenv("ROBOT_BUS_NATIVE") ?: System.getProperty("robot.bus.native")
        if (!explicit.isNullOrBlank()) {
            return Native.load(explicit, RobotBusC::class.java)
        }
        val dir = System.getenv("ROBOT_BUS_NATIVE_DIR") ?: System.getProperty("robot.bus.native.dir")
        val candidates = buildList {
            if (!dir.isNullOrBlank()) add(Path.of(dir))
            add(Path.of("").toAbsolutePath().resolve("../cpp/native/target/release").normalize())
            add(Path.of(System.getProperty("user.dir"), "bindings/cpp/native/target/release"))
            // When cwd is bindings/kotlin
            add(Path.of(System.getProperty("user.dir"), "../cpp/native/target/release").normalize())
        }
        for (candidate in candidates) {
            val file = findInDir(candidate) ?: continue
            return Native.load(file.toAbsolutePath().toString(), RobotBusC::class.java)
        }
        return Native.load("robot_bus_c", RobotBusC::class.java)
    }

    private fun findInDir(dir: Path): Path? {
        if (!Files.isDirectory(dir)) return null
        val names = when {
            Platform.isMac() -> listOf("librobot_bus_c.dylib", "librobot_bus.dylib")
            Platform.isWindows() -> listOf("robot_bus_c.dll", "robot_bus.dll")
            else -> listOf("librobot_bus_c.so", "librobot_bus.so")
        }
        return names.map { dir.resolve(it) }.firstOrNull { Files.isRegularFile(it) }
    }
}
