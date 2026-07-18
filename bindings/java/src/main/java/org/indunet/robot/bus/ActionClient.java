package org.indunet.robot.bus;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/** Client for a named action on a {@link Node}. */
public final class ActionClient implements AutoCloseable {
    private Pointer ptr;

    ActionClient(Pointer ptr) {
        this.ptr = ptr;
    }

    public String actionName() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_action_client_action_name(ptr));
    }

    public List<ActionMessage> sendGoal(byte[] body) {
        return sendGoal(body, null, -1.0);
    }

    public List<ActionMessage> sendGoal(byte[] body, String goalId) {
        return sendGoal(body, goalId, -1.0);
    }

    public List<ActionMessage> sendGoal(byte[] body, String goalId, double timeoutSecs) {
        PointerByReference outMsgs = new PointerByReference();
        LongByReference outCount = new LongByReference();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_action_client_send_goal(
                        ptr, body, body.length, goalId, timeoutSecs, outMsgs, outCount),
                "send_goal");
        long count = outCount.getValue();
        Pointer base = outMsgs.getValue();
        if (base == null || count <= 0) {
            return Collections.emptyList();
        }
        try {
            long size = new RobotBusC.ActionMessageStruct().size();
            List<ActionMessage> result = new ArrayList<>((int) count);
            for (long i = 0; i < count; i++) {
                RobotBusC.ActionMessageStruct msg = new RobotBusC.ActionMessageStruct(base.share(i * size));
                String kind = msg.kind != null ? msg.kind.getString(0) : "";
                byte[] bodyBytes =
                        (msg.body != null && msg.bodyLen > 0)
                                ? msg.body.getByteArray(0, (int) msg.bodyLen)
                                : new byte[0];
                String gid = msg.goalId != null ? msg.goalId.getString(0) : "";
                String name = msg.actionName != null ? msg.actionName.getString(0) : "";
                result.add(new ActionMessage(kind, bodyBytes, gid, name));
            }
            return result;
        } finally {
            RobotBusC.Holder.INSTANCE.robot_bus_action_messages_free(base, count);
        }
    }

    public ActionMessage cancel(String goalId) {
        return cancel(goalId, new byte[0], -1.0);
    }

    public ActionMessage cancel(String goalId, byte[] body) {
        return cancel(goalId, body, -1.0);
    }

    public ActionMessage cancel(String goalId, byte[] body, double timeoutSecs) {
        RobotBusC.ActionMessageStruct out = new RobotBusC.ActionMessageStruct();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_action_client_cancel(
                        ptr, goalId, body, body.length, timeoutSecs, out),
                "cancel");
        out.read();
        String kind = out.kind != null ? out.kind.getString(0) : "";
        byte[] bodyBytes =
                (out.body != null && out.bodyLen > 0)
                        ? out.body.getByteArray(0, (int) out.bodyLen)
                        : new byte[0];
        String gid = out.goalId != null ? out.goalId.getString(0) : "";
        String name = out.actionName != null ? out.actionName.getString(0) : "";
        ActionMessage result = new ActionMessage(kind, bodyBytes, gid, name);
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(out.kind);
        RobotBusC.Holder.INSTANCE.robot_bus_free_bytes(out.body, out.bodyLen);
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(out.goalId);
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(out.actionName);
        return result;
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_action_client_free(ptr);
        ptr = null;
    }
}
