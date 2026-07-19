package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/**
 * Shared ZeroMQ runtime context. Same-process inproc between an embedded {@link Broker} and
 * {@link Node}s requires one shared {@code Context}.
 */
public final class Context implements AutoCloseable {
    private Pointer ptr;

    public Context() {
        this.ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_context_new(), "Context");
    }

    private Context(Pointer ptr) {
        this.ptr = ptr;
    }

    /** Cheap clone (refcounted ZMQ context). */
    public Context cloneContext() {
        return new Context(
                Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_context_clone(ptr), "Context.clone"));
    }

    Pointer raw() {
        return ptr;
    }

    @Override
    public void close() {
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_context_free(ptr);
            ptr = null;
        }
    }
}
