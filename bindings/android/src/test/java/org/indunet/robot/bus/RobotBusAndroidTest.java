package org.indunet.robot.bus;

import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

import org.junit.Test;

/**
 * Android-module smoke tests. Full gRPC / typed Node coverage lives in the JVM
 * artifact ({@code bindings/java} {@code GrpcNodeTest} / {@code TypedApiTest});
 * this AAR only adds {@link RobotBusAndroid#init} + jniLibs.
 */
public class RobotBusAndroidTest {
    @Test
    public void initFailsWithoutJniLibsOnHostJvm() {
        try {
            // Context is unused today; host JVM has no librobot_bus_c in
            // java.library.path the way APK jniLibs packaging provides it.
            RobotBusAndroid.init(null);
            fail("expected UnsatisfiedLinkError without Android jniLibs");
        } catch (UnsatisfiedLinkError expected) {
            assertTrue(expected.getMessage() == null || expected.getMessage().contains("robot_bus_c"));
        }
    }
}
