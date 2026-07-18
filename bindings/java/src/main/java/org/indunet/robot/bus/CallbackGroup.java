package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Groups callbacks for concurrency control. */
public final class CallbackGroup implements AutoCloseable {
    private Pointer ptr;

    CallbackGroup(Pointer ptr) {
        this.ptr = ptr;
    }

    public long id() {
        return RobotBusC.Holder.INSTANCE.robot_bus_callback_group_id(ptr);
    }

    public CallbackGroupType kind() {
        int code = RobotBusC.Holder.INSTANCE.robot_bus_callback_group_kind(ptr);
        if (code == 1) {
            return CallbackGroupType.Reentrant;
        }
        return CallbackGroupType.MutuallyExclusive;
    }

    Pointer raw() {
        return ptr;
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_callback_group_free(ptr);
        ptr = null;
    }
}
