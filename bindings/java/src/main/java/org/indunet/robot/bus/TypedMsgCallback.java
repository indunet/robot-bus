package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;

/** Callback for typed subscription messages. */
@FunctionalInterface
public interface TypedMsgCallback<T extends MessageLite> {
    void onMessage(T message);
}
