package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;
import java.util.List;

/** Handler for typed action goals; returns phases to publish. */
@FunctionalInterface
public interface TypedActionHandler<Goal extends MessageLite> {
    List<TypedActionPhase> handle(Goal goal);
}
