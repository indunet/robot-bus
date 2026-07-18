package org.indunet.robot.bus;

import com.sun.jna.Pointer;

final class Errors {
    private Errors() {}

    static String lastError() {
        String err = RobotBusC.Holder.INSTANCE.robot_bus_last_error();
        return err != null ? err : "";
    }

    static void check(int rc, String what) {
        if (rc < 0) {
            String err = lastError();
            throw new RobotBusException(what + ": " + (err.isEmpty() ? "unknown error" : err));
        }
    }

    static Pointer checkPtr(Pointer p, String what) {
        if (p == null) {
            String err = lastError();
            throw new RobotBusException(what + ": " + (err.isEmpty() ? "null" : err));
        }
        return p;
    }
}
