package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Publisher bound to a topic on a {@link Node}. */
public final class TopicPublisher implements AutoCloseable {
    private Pointer ptr;

    TopicPublisher(Pointer ptr) {
        this.ptr = ptr;
    }

    public String topic() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_topic_publisher_topic(ptr));
    }

    public void publish(byte[] payload) {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_topic_publisher_publish(ptr, payload, payload.length),
                "publish");
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_topic_publisher_free(ptr);
        ptr = null;
    }
}
