package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Opaque timer handle returned by {@link Node#createTimer}. */
public final class TimerHandle implements AutoCloseable {
    final Pointer ptr;

    TimerHandle(Pointer ptr) {
        this.ptr = ptr;
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_timer_handle_free(ptr);
    }
}
