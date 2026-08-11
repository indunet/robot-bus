package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Opaque action server handle returned by {@link Node#createActionServer}. */
public final class ActionServerHandle implements AutoCloseable {
    private final Pointer node;
    Pointer ptr;
    private boolean destroyed;

    ActionServerHandle(Pointer node, Pointer ptr) {
        this.node = node;
        this.ptr = ptr;
    }

    public String actionName() {
        if (ptr == null) {
            return "";
        }
        Pointer s = RobotBusC.Holder.INSTANCE.robot_bus_action_server_handle_name(ptr);
        return NativeUtils.takeCString(s);
    }

    public void destroy() {
        if (destroyed || ptr == null) {
            return;
        }
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_destroy_action_server(node, ptr),
                "destroy_action_server");
        destroyed = true;
    }

    @Override
    public void close() {
        destroy();
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_action_server_handle_free(ptr);
            ptr = null;
        }
    }
}
