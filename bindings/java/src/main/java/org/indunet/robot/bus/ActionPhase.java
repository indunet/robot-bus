package org.indunet.robot.bus;

import java.util.Arrays;
import java.util.Objects;

/** One phase emitted by an action server handler. */
public final class ActionPhase {
    private final String phase;
    private final byte[] body;

    public ActionPhase(String phase, byte[] body) {
        this.phase = phase != null ? phase : "";
        this.body = body != null ? body : new byte[0];
    }

    public String getPhase() {
        return phase;
    }

    public byte[] getBody() {
        return body;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof ActionPhase)) {
            return false;
        }
        ActionPhase that = (ActionPhase) other;
        return Objects.equals(phase, that.phase) && Arrays.equals(body, that.body);
    }

    @Override
    public int hashCode() {
        return 31 * Objects.hashCode(phase) + Arrays.hashCode(body);
    }
}
