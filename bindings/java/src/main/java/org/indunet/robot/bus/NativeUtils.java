package org.indunet.robot.bus;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.PointerByReference;

final class NativeUtils {
    private NativeUtils() {}

    static String takeCString(Pointer p) {
        if (p == null) {
            return "";
        }
        try {
            return p.getString(0);
        } finally {
            RobotBusC.Holder.INSTANCE.robot_bus_free_string(p);
        }
    }

    static byte[] readBytes(Pointer p, long len) {
        if (p == null || len <= 0) {
            return new byte[0];
        }
        try {
            return p.getByteArray(0, (int) len);
        } finally {
            RobotBusC.Holder.INSTANCE.robot_bus_free_bytes(p, len);
        }
    }

    static Pointer allocReplyBytes(byte[] payload) {
        if (payload == null || payload.length == 0) {
            return null;
        }
        Pointer buf = RobotBusC.Holder.INSTANCE.robot_bus_alloc_bytes(payload.length);
        if (buf == null) {
            throw new RobotBusException("robot_bus_alloc_bytes failed");
        }
        buf.write(0, payload, 0, payload.length);
        return buf;
    }

    @FunctionalInterface
    interface EndpointBlock {
        int call(PointerByReference out);
    }

    static String endpointCall(EndpointBlock block) {
        PointerByReference out = new PointerByReference();
        Errors.check(block.call(out), "endpoint");
        return takeCString(out.getValue());
    }
}
