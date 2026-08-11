package org.indunet.robot.bus;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;

/** Client for a named service on a {@link Node}. */
public final class ServiceClient implements AutoCloseable {
    private Pointer ptr;

    ServiceClient(Pointer ptr) {
        this.ptr = ptr;
    }

    public String serviceName() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_service_client_service_name(ptr));
    }

    public boolean serviceIsReady() {
        return RobotBusC.Holder.INSTANCE.robot_bus_service_client_service_is_ready(ptr) != 0;
    }

    public boolean waitForService() {
        return waitForService(-1.0);
    }

    public boolean waitForService(double timeoutSecs) {
        return RobotBusC.Holder.INSTANCE.robot_bus_service_client_wait_for_service(ptr, timeoutSecs) != 0;
    }

    public byte[] call(byte[] body) {
        return call(body, -1.0);
    }

    public byte[] call(byte[] body, double timeoutSecs) {
        PointerByReference outData = new PointerByReference();
        LongByReference outLen = new LongByReference();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_service_client_call(
                        ptr, body, body.length, timeoutSecs, outData, outLen),
                "service call");
        return NativeUtils.readBytes(outData.getValue(), outLen.getValue());
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_service_client_free(ptr);
        ptr = null;
    }
}
