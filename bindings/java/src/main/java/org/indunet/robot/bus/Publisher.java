package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Standalone pub socket (not tied to a {@link Node}). */
public final class Publisher implements AutoCloseable {
    private Pointer ptr;

    public Publisher() {
        this((String) null);
    }

    public Publisher(String endpoint) {
        this.ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_publisher_new(endpoint), "Publisher");
    }

    public void publish(String topic, byte[] payload) {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_publisher_publish(
                        ptr, topic, payload, payload.length),
                "publish");
    }

    public String endpoint() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_publisher_endpoint(ptr));
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_publisher_free(ptr);
        ptr = null;
    }
}
