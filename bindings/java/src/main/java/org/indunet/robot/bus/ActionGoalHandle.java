package org.indunet.robot.bus;

import com.sun.jna.Pointer;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.locks.ReentrantReadWriteLock;
import java.util.function.Consumer;

/** Owns one in-flight action goal and its native feedback callback. */
public final class ActionGoalHandle implements AutoCloseable {
    private Pointer ptr;
    // JNA callbacks must remain strongly reachable for as long as native code may invoke them.
    @SuppressWarnings("unused")
    private final RobotBusC.ActionFeedbackCb feedbackCallback;
    private final ReentrantReadWriteLock lifecycle = new ReentrantReadWriteLock();

    ActionGoalHandle(Pointer ptr, RobotBusC.ActionFeedbackCb feedbackCallback) {
        this.ptr = Objects.requireNonNull(ptr, "ptr");
        this.feedbackCallback = feedbackCallback;
    }

    static RobotBusC.ActionFeedbackCb feedbackCallback(Consumer<ActionMessage> feedback) {
        if (feedback == null) {
            return null;
        }
        return (message, user) -> {
            if (message != null) {
                feedback.accept(copyMessage(new RobotBusC.ActionMessageStruct(message)));
            }
        };
    }

    public String goalId() {
        lifecycle.readLock().lock();
        try {
            return NativeUtils.takeCString(
                    RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_goal_id(requireOpen()));
        } finally {
            lifecycle.readLock().unlock();
        }
    }

    public String actionName() {
        lifecycle.readLock().lock();
        try {
            return NativeUtils.takeCString(
                    RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_action_name(requireOpen()));
        } finally {
            lifecycle.readLock().unlock();
        }
    }

    public ActionMessage result() {
        return result(-1.0);
    }

    public ActionMessage result(double timeoutSecs) {
        lifecycle.readLock().lock();
        try {
            RobotBusC.ActionMessageStruct out = new RobotBusC.ActionMessageStruct();
            Errors.check(
                    RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_wait_result(
                            requireOpen(), timeoutSecs, out),
                    "action_goal_result");
            out.read();
            try {
                return copyMessage(out);
            } finally {
                freeMessage(out);
            }
        } finally {
            lifecycle.readLock().unlock();
        }
    }

    public CompletableFuture<ActionMessage> resultAsync() {
        return resultAsync(-1.0);
    }

    public CompletableFuture<ActionMessage> resultAsync(double timeoutSecs) {
        return CompletableFuture.supplyAsync(() -> result(timeoutSecs));
    }

    public void cancel() {
        lifecycle.readLock().lock();
        try {
            Errors.check(
                    RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_cancel(requireOpen()),
                    "action_goal_cancel");
        } finally {
            lifecycle.readLock().unlock();
        }
    }

    @Override
    public void close() {
        lifecycle.writeLock().lock();
        try {
            if (ptr != null) {
                RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_free(ptr);
                ptr = null;
            }
        } finally {
            lifecycle.writeLock().unlock();
        }
    }

    private Pointer requireOpen() {
        if (ptr == null) {
            throw new IllegalStateException("action goal handle is closed");
        }
        return ptr;
    }

    private static ActionMessage copyMessage(RobotBusC.ActionMessageStruct msg) {
        String kind = msg.kind != null ? msg.kind.getString(0) : "";
        byte[] body =
                msg.body != null && msg.bodyLen > 0
                        ? msg.body.getByteArray(0, (int) msg.bodyLen)
                        : new byte[0];
        String goalId = msg.goalId != null ? msg.goalId.getString(0) : "";
        String actionName = msg.actionName != null ? msg.actionName.getString(0) : "";
        return new ActionMessage(kind, body, goalId, actionName);
    }

    private static void freeMessage(RobotBusC.ActionMessageStruct msg) {
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(msg.kind);
        RobotBusC.Holder.INSTANCE.robot_bus_free_bytes(msg.body, msg.bodyLen);
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(msg.goalId);
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(msg.actionName);
    }
}
