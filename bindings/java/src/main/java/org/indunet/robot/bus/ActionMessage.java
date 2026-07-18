package org.indunet.robot.bus;

import java.util.Arrays;
import java.util.Objects;

/** One message in an action goal stream (feedback / result / etc.). */
public final class ActionMessage {
    private final String kind;
    private final byte[] body;
    private final String goalId;
    private final String actionName;

    public ActionMessage(String kind, byte[] body, String goalId, String actionName) {
        this.kind = kind != null ? kind : "";
        this.body = body != null ? body : new byte[0];
        this.goalId = goalId != null ? goalId : "";
        this.actionName = actionName != null ? actionName : "";
    }

    public String getKind() {
        return kind;
    }

    public byte[] getBody() {
        return body;
    }

    public String getGoalId() {
        return goalId;
    }

    public String getActionName() {
        return actionName;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof ActionMessage)) {
            return false;
        }
        ActionMessage that = (ActionMessage) other;
        return Objects.equals(kind, that.kind)
                && Arrays.equals(body, that.body)
                && Objects.equals(goalId, that.goalId)
                && Objects.equals(actionName, that.actionName);
    }

    @Override
    public int hashCode() {
        int result = Objects.hashCode(kind);
        result = 31 * result + Arrays.hashCode(body);
        result = 31 * result + Objects.hashCode(goalId);
        result = 31 * result + Objects.hashCode(actionName);
        return result;
    }
}
