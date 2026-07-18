package org.indunet.robot.bus;

/** Unchecked error from the robot-bus native layer. */
public class RobotBusException extends RuntimeException {
    public RobotBusException(String message) {
        super(message);
    }
}
