package org.indunet.robot.bus;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;

/** Standalone sub socket (not tied to a {@link Node}). */
public final class Subscriber implements AutoCloseable {
    private Pointer ptr;

    public Subscriber() {
        this((String) null);
    }

    public Subscriber(String endpoint) {
        this.ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_subscriber_new(endpoint), "Subscriber");
    }

    public void subscribe(String topic) {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_subscriber_subscribe(ptr, topic), "subscribe");
    }

    public void unsubscribe(String topic) {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_subscriber_unsubscribe(ptr, topic), "unsubscribe");
    }

    /**
     * Receive one message.
     *
     * @param timeoutSecs negative blocks until a message arrives; use {@link #receive()} for that default
     */
    public TopicMessage receive(double timeoutSecs) {
        PointerByReference outTopic = new PointerByReference();
        PointerByReference outData = new PointerByReference();
        LongByReference outLen = new LongByReference();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_subscriber_receive(
                        ptr, timeoutSecs, outTopic, outData, outLen),
                "receive");
        String topic = NativeUtils.takeCString(outTopic.getValue());
        byte[] payload = NativeUtils.readBytes(outData.getValue(), outLen.getValue());
        return new TopicMessage(topic, payload);
    }

    /** Blocks until a message arrives. */
    public TopicMessage receive() {
        return receive(-1.0);
    }

    public String endpoint() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_subscriber_endpoint(ptr));
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_subscriber_free(ptr);
        ptr = null;
    }
}
