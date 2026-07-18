package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;

/** Publisher that accepts protobuf message instances. */
public final class TypedTopicPublisher<T extends MessageLite> implements AutoCloseable {
    private final TopicPublisher inner;
    private final Class<T> msgType;

    TypedTopicPublisher(TopicPublisher inner, Class<T> msgType) {
        this.inner = inner;
        this.msgType = msgType;
    }

    public String topic() {
        return inner.topic();
    }

    public Class<T> msgType() {
        return msgType;
    }

    public void publish(T msg) {
        if (msg == null) {
            throw new NullPointerException("msg");
        }
        if (!msgType.isInstance(msg)) {
            throw new IllegalArgumentException(
                    "publisher for " + msgType.getSimpleName() + " got " + msg.getClass().getSimpleName());
        }
        inner.publish(ProtoCodec.encode(msg));
    }

    @Override
    public void close() {
        inner.close();
    }

    @Override
    public String toString() {
        return "TypedTopicPublisher{topic=" + topic() + ", msgType=" + msgType.getSimpleName() + '}';
    }
}
