package org.indunet.robot.bus;

import org.indunet.robot.bus.geometry_msgs.msg.v1.TransformStamped;
import org.indunet.robot.bus.tf2_msgs.msg.v1.TFMessage;

/** Thin helper over a typed {@code TFMessage} publisher. */
public final class TransformBroadcaster implements AutoCloseable {
    private final TypedTopicPublisher<TFMessage> publisher;

    public TransformBroadcaster(TypedTopicPublisher<TFMessage> publisher) {
        if (publisher == null) {
            throw new NullPointerException("publisher");
        }
        this.publisher = publisher;
    }

    public void send(TFMessage msg) {
        publisher.publish(msg);
    }

    public void send(TransformStamped... transforms) {
        TFMessage.Builder b = TFMessage.newBuilder();
        for (TransformStamped t : transforms) {
            b.addTransforms(t);
        }
        publisher.publish(b.build());
    }

    @Override
    public void close() {
        publisher.close();
    }
}
