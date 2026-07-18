package org.indunet.robot.bus;

/** Handler for service requests. */
@FunctionalInterface
public interface ServiceHandler {
    byte[] handle(byte[] body);
}
