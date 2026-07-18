package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;
import java.util.ArrayList;
import java.util.List;

/** Action client that encodes goals and decodes FEEDBACK / RESULT bodies. */
public final class TypedActionClient<
                Goal extends MessageLite, Feedback extends MessageLite, Result extends MessageLite>
        implements AutoCloseable {
    private final ActionClient inner;
    private final Class<Goal> goalType;
    private final Class<Feedback> feedbackType;
    private final Class<Result> resultType;

    TypedActionClient(
            ActionClient inner,
            Class<Goal> goalType,
            Class<Feedback> feedbackType,
            Class<Result> resultType) {
        this.inner = inner;
        this.goalType = goalType;
        this.feedbackType = feedbackType;
        this.resultType = resultType;
    }

    public String actionName() {
        return inner.actionName();
    }

    public Class<Goal> goalType() {
        return goalType;
    }

    public Class<Feedback> feedbackType() {
        return feedbackType;
    }

    public Class<Result> resultType() {
        return resultType;
    }

    public List<TypedActionMessage> sendGoal(Goal goal) {
        return sendGoal(goal, null, -1.0);
    }

    public List<TypedActionMessage> sendGoal(Goal goal, String goalId) {
        return sendGoal(goal, goalId, -1.0);
    }

    public List<TypedActionMessage> sendGoal(Goal goal, String goalId, double timeoutSecs) {
        if (goal == null) {
            throw new NullPointerException("goal");
        }
        if (!goalType.isInstance(goal)) {
            throw new IllegalArgumentException(
                    "action client for "
                            + goalType.getSimpleName()
                            + " got "
                            + goal.getClass().getSimpleName());
        }
        List<ActionMessage> raw = inner.sendGoal(ProtoCodec.encode(goal), goalId, timeoutSecs);
        List<TypedActionMessage> out = new ArrayList<>(raw.size());
        for (ActionMessage msg : raw) {
            TypedActionMessage decoded = decode(msg);
            if (decoded.getBody() == null
                    && isTypedKind(msg.getKind())
                    && msg.getBody().length > 0) {
                continue;
            }
            out.add(decoded);
        }
        return out;
    }

    public TypedActionMessage cancel(String goalId) {
        return cancel(goalId, null, -1.0);
    }

    public TypedActionMessage cancel(String goalId, MessageLite body) {
        return cancel(goalId, body, -1.0);
    }

    public TypedActionMessage cancel(String goalId, MessageLite body, double timeoutSecs) {
        byte[] raw = body == null ? new byte[0] : ProtoCodec.encode(body);
        return decode(inner.cancel(goalId, raw, timeoutSecs));
    }

    private TypedActionMessage decode(ActionMessage msg) {
        String kind = msg.getKind();
        MessageLite decoded = null;
        if ("FEEDBACK".equalsIgnoreCase(kind)) {
            decoded = ProtoCodec.tryParse(feedbackType, msg.getBody());
        } else if ("RESULT".equalsIgnoreCase(kind)) {
            decoded = ProtoCodec.tryParse(resultType, msg.getBody());
        } else if ("GOAL".equalsIgnoreCase(kind)) {
            decoded = ProtoCodec.tryParse(goalType, msg.getBody());
        }
        return new TypedActionMessage(kind, decoded, msg.getBody(), msg.getGoalId(), msg.getActionName());
    }

    private static boolean isTypedKind(String kind) {
        return "FEEDBACK".equalsIgnoreCase(kind)
                || "RESULT".equalsIgnoreCase(kind)
                || "GOAL".equalsIgnoreCase(kind);
    }

    @Override
    public void close() {
        inner.close();
    }

    @Override
    public String toString() {
        return "TypedActionClient{action="
                + actionName()
                + ", goal="
                + goalType.getSimpleName()
                + '}';
    }
}
