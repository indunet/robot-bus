package org.indunet.robot.bus;

import java.util.Arrays;
import java.util.Objects;

/** A topic name plus payload bytes. */
public final class TopicMessage {
    private final String topic;
    private final byte[] payload;

    public TopicMessage(String topic, byte[] payload) {
        this.topic = topic != null ? topic : "";
        this.payload = payload != null ? payload : new byte[0];
    }

    public String getTopic() {
        return topic;
    }

    public byte[] getPayload() {
        return payload;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof TopicMessage)) {
            return false;
        }
        TopicMessage that = (TopicMessage) other;
        return Objects.equals(topic, that.topic) && Arrays.equals(payload, that.payload);
    }

    @Override
    public int hashCode() {
        return 31 * Objects.hashCode(topic) + Arrays.hashCode(payload);
    }
}
