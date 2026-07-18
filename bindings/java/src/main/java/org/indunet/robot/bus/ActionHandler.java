package org.indunet.robot.bus;

import java.util.List;

/** Handler for action goals; returns phases to publish. */
@FunctionalInterface
public interface ActionHandler {
    List<ActionPhase> handle(byte[] body);
}
