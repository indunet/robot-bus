package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Opaque service server handle returned by {@link Node#createService}. */
public final class ServiceHandle implements AutoCloseable {
    private final Pointer node;
    Pointer ptr;
    private boolean destroyed;

    ServiceHandle(Pointer node, Pointer ptr) {
        this.node = node;
        this.ptr = ptr;
    }

    public String serviceName() {
        if (ptr == null) {
            return "";
        }
        Pointer s = RobotBusC.Holder.INSTANCE.robot_bus_service_handle_name(ptr);
        return NativeUtils.takeCString(s);
    }

    public void destroy() {
        if (destroyed || ptr == null) {
            return;
        }
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_destroy_service(node, ptr),
                "destroy_service");
        destroyed = true;
    }

    @Override
    public void close() {
        destroy();
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_service_handle_free(ptr);
            ptr = null;
        }
    }
}
