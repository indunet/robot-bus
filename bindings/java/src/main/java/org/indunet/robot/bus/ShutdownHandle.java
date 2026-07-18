package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Handle used to request shutdown from another thread. */
public final class ShutdownHandle implements AutoCloseable {
    private Pointer ptr;

    ShutdownHandle(Pointer ptr) {
        this.ptr = ptr;
    }

    public void shutdown() {
        RobotBusC.Holder.INSTANCE.robot_bus_shutdown_handle_shutdown(ptr);
    }

    public boolean isRunning() {
        return RobotBusC.Holder.INSTANCE.robot_bus_shutdown_handle_is_running(ptr) != 0;
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_shutdown_handle_free(ptr);
        ptr = null;
    }
}
