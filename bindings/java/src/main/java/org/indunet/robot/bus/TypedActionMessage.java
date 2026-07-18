package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;
import java.util.Objects;

/**
 * Action stream message with a decoded protobuf body when the kind matches the
 * bound goal / feedback / result types; otherwise {@link #getBody()} is null and
 * {@link #getRawBody()} holds the opaque bytes.
 */
public final class TypedActionMessage {
    private final String kind;
    private final MessageLite body;
    private final byte[] rawBody;
    private final String goalId;
    private final String actionName;

    TypedActionMessage(String kind, MessageLite body, byte[] rawBody, String goalId, String actionName) {
        this.kind = kind != null ? kind : "";
        this.body = body;
        this.rawBody = rawBody != null ? rawBody : new byte[0];
        this.goalId = goalId != null ? goalId : "";
        this.actionName = actionName != null ? actionName : "";
    }

    public String getKind() {
        return kind;
    }

    /** Decoded protobuf body, or {@code null} if decode was skipped / failed. */
    public MessageLite getBody() {
        return body;
    }

    public byte[] getRawBody() {
        return rawBody;
    }

    public String getGoalId() {
        return goalId;
    }

    public String getActionName() {
        return actionName;
    }

    @Override
    public String toString() {
        return "TypedActionMessage{kind="
                + kind
                + ", body="
                + (body != null ? body.getClass().getSimpleName() : "null")
                + ", goalId="
                + goalId
                + '}';
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof TypedActionMessage)) {
            return false;
        }
        TypedActionMessage that = (TypedActionMessage) other;
        return Objects.equals(kind, that.kind)
                && Objects.equals(body, that.body)
                && Objects.equals(goalId, that.goalId)
                && Objects.equals(actionName, that.actionName);
    }

    @Override
    public int hashCode() {
        return Objects.hash(kind, body, goalId, actionName);
    }
}
