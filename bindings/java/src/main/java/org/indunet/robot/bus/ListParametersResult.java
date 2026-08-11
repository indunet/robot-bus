package org.indunet.robot.bus;

import java.util.Collections;
import java.util.List;

/** ROS-shaped result of {@link Node#listParameters}. */
public final class ListParametersResult {
    private final List<String> names;
    private final List<String> prefixes;

    public ListParametersResult(List<String> names, List<String> prefixes) {
        this.names = names != null ? names : Collections.emptyList();
        this.prefixes = prefixes != null ? prefixes : Collections.emptyList();
    }

    public List<String> getNames() {
        return names;
    }

    public List<String> getPrefixes() {
        return prefixes;
    }
}
