package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Subscribes {@code /tf} + {@code /tf_static} (or custom topics) into a shared buffer. */
public final class TfListener implements AutoCloseable {
    private Pointer ptr;

    public TfListener(Node node) {
        this.ptr =
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_with_defaults(node.raw()),
                        "TfListener");
    }

    public TfListener(Node node, String tfTopic, String tfStaticTopic) {
        this.ptr =
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_new(
                                node.raw(), tfTopic, tfStaticTopic),
                        "TfListener");
    }

    /** Shared buffer handle (Arc clone). Safe to close independently of this listener. */
    public TfBuffer buffer() {
        return new TfBuffer(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_buffer(ptr),
                        "TfListener.buffer"));
    }

    @Override
    public void close() {
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_free(ptr);
            ptr = null;
        }
    }
}
