package org.indunet.robot.bus;

import com.google.protobuf.InvalidProtocolBufferException;
import com.google.protobuf.MessageLite;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.logging.Level;
import java.util.logging.Logger;

/** Encode / decode helpers for typed Node APIs (mirrors Python {@code _typed.py}). */
final class ProtoCodec {
    private static final Logger LOG = Logger.getLogger("org.indunet.robot.bus");

    private ProtoCodec() {}

    static byte[] encode(MessageLite msg) {
        if (msg == null) {
            throw new NullPointerException("expected a protobuf MessageLite");
        }
        return msg.toByteArray();
    }

    @SuppressWarnings("unchecked")
    static <T extends MessageLite> T parse(Class<T> type, byte[] payload) {
        try {
            Method parseFrom = type.getMethod("parseFrom", byte[].class);
            return (T) parseFrom.invoke(null, payload != null ? payload : new byte[0]);
        } catch (InvocationTargetException e) {
            Throwable cause = e.getCause();
            if (cause instanceof RuntimeException) {
                throw (RuntimeException) cause;
            }
            if (cause instanceof InvalidProtocolBufferException) {
                throw new IllegalArgumentException(
                        "invalid protobuf payload for " + type.getSimpleName(), cause);
            }
            throw new IllegalStateException("parseFrom failed for " + type.getName(), cause);
        } catch (NoSuchMethodException | IllegalAccessException e) {
            throw new IllegalArgumentException(
                    type.getName() + " is not a generated protobuf message (missing parseFrom)", e);
        }
    }

    static <T extends MessageLite> T tryParse(Class<T> type, byte[] payload) {
        try {
            return parse(type, payload);
        } catch (Exception err) {
            LOG.log(Level.WARNING, "typed decode failed for " + type.getSimpleName() + ": " + err, err);
            return null;
        }
    }

    static void requireMessageType(Class<?> type, String what) {
        if (type == null || !MessageLite.class.isAssignableFrom(type)) {
            throw new IllegalArgumentException(
                    what + " must be a com.google.protobuf.MessageLite subclass, got " + type);
        }
    }
}
