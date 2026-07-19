package org.indunet.robot.bus;

/** A named local node parameter. */
public final class Parameter {
    private final String name;
    private final Object value;

    public Parameter(String name, Object value) {
        this.name = name;
        this.value = value;
    }

    public String getName() {
        return name;
    }

    /** {@link Boolean}, {@link Long}, {@link Double}, or {@link String}. */
    public Object getValue() {
        return value;
    }
}
