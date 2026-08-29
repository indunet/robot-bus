package org.indunet.robot.bus;

/** Callback for subscription messages. */
@FunctionalInterface
public interface MsgCallback {
    void onMessage(byte[] payload);
}
