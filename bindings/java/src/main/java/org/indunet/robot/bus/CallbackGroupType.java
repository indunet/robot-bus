package org.indunet.robot.bus;

/** Callback-group concurrency kind (maps to C enum). */
public enum CallbackGroupType {
    MutuallyExclusive(0),
    Reentrant(1);

    private final int code;

    CallbackGroupType(int code) {
        this.code = code;
    }

    public int getCode() {
        return code;
    }
}
