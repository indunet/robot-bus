package org.indunet.robot.bus

import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

/**
 * Android entry smoke tests. Parameter / typed API coverage uses desktop
 * `librobot_bus_c` via `ROBOT_BUS_NATIVE_DIR` (see [NodeParametersTest], [TypedApiTest]).
 */
class RobotBusAndroidTest {
    @Test
    fun initFailsWithoutJniLibsOnHostJvm() {
        try {
            // Context is unused today; host JVM has no librobot_bus_c in
            // java.library.path the way APK jniLibs packaging provides it.
            RobotBusAndroid.init(null)
            fail("expected UnsatisfiedLinkError without Android jniLibs")
        } catch (expected: UnsatisfiedLinkError) {
            assertTrue(
                expected.message == null || expected.message!!.contains("robot_bus_c"),
            )
        }

        // Failure must not mark initialized; a second call should still fail the same way.
        try {
            RobotBusAndroid.init(null)
            fail("expected UnsatisfiedLinkError on retry after failed init")
        } catch (expected: UnsatisfiedLinkError) {
            assertTrue(
                expected.message == null || expected.message!!.contains("robot_bus_c"),
            )
        }
    }
}
