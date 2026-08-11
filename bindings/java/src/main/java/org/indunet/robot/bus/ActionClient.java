package org.indunet.robot.bus;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.PointerByReference;
import java.util.Objects;
import java.util.function.Consumer;

/** Client for a named action on a {@link Node}. */
public final class ActionClient implements AutoCloseable {
    private Pointer ptr;

    ActionClient(Pointer ptr) {
        this.ptr = ptr;
    }

    public String actionName() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_action_client_action_name(ptr));
    }

    public boolean actionServerIsReady() {
        return RobotBusC.Holder.INSTANCE.robot_bus_action_client_action_server_is_ready(ptr) != 0;
    }

    public boolean waitForActionServer() {
        return waitForActionServer(-1.0);
    }

    public boolean waitForActionServer(double timeoutSecs) {
        return RobotBusC.Holder.INSTANCE.robot_bus_action_client_wait_for_action_server(ptr, timeoutSecs)
                != 0;
    }

    public ActionGoalHandle sendGoal(byte[] body) {
        return sendGoal(body, null, -1.0, null);
    }

    public ActionGoalHandle sendGoal(byte[] body, Consumer<ActionMessage> feedback) {
        return sendGoal(body, null, -1.0, feedback);
    }

    public ActionGoalHandle sendGoal(byte[] body, String goalId) {
        return sendGoal(body, goalId, -1.0, null);
    }

    public ActionGoalHandle sendGoal(byte[] body, String goalId, double timeoutSecs) {
        return sendGoal(body, goalId, timeoutSecs, null);
    }

    public ActionGoalHandle sendGoal(
            byte[] body,
            String goalId,
            double timeoutSecs,
            Consumer<ActionMessage> feedback) {
        Objects.requireNonNull(body, "body");
        RobotBusC.ActionFeedbackCb callback = ActionGoalHandle.feedbackCallback(feedback);
        PointerByReference outHandle = new PointerByReference();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_action_client_send_goal(
                        ptr,
                        body,
                        body.length,
                        goalId,
                        timeoutSecs,
                        callback,
                        null,
                        outHandle),
                "send_goal");
        Pointer handle = Errors.checkPtr(outHandle.getValue(), "send_goal handle");
        return new ActionGoalHandle(handle, callback);
    }

    @Override
    public void close() {
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_action_client_free(ptr);
            ptr = null;
        }
    }
}
