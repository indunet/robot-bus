package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;
import java.util.Objects;

/** One phase emitted by a typed action server handler. */
public final class TypedActionPhase {
    private final String phase;
    private final MessageLite body;

    public TypedActionPhase(String phase, MessageLite body) {
        this.phase = phase != null ? phase : "";
        this.body = Objects.requireNonNull(body, "body");
    }

    public String getPhase() {
        return phase;
    }

    public MessageLite getBody() {
        return body;
    }
}
