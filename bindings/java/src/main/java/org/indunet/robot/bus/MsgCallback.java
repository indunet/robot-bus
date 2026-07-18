package org.indunet.robot.bus;

/** Callback for subscription messages. */
@FunctionalInterface
public interface MsgCallback {
    void onMessage(String topic, byte[] payload);
}
