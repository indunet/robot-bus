package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Opaque subscription handle returned by {@link Node#createSubscription}. */
public final class SubscriptionHandle implements AutoCloseable {
    private final Pointer node;
    Pointer ptr;
    private boolean destroyed;

    SubscriptionHandle(Pointer node, Pointer ptr) {
        this.node = node;
        this.ptr = ptr;
    }

    public void destroy() {
        if (destroyed || ptr == null) {
            return;
        }
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_destroy_subscription(node, ptr),
                "destroy_subscription");
        destroyed = true;
    }

    @Override
    public void close() {
        destroy();
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_subscription_handle_free(ptr);
            ptr = null;
        }
    }
}
