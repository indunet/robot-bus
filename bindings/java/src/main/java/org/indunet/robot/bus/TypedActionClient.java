package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;
import java.util.function.Consumer;

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

    public boolean actionServerIsReady() {
        return inner.actionServerIsReady();
    }

    public boolean waitForActionServer() {
        return inner.waitForActionServer();
    }

    public boolean waitForActionServer(double timeoutSecs) {
        return inner.waitForActionServer(timeoutSecs);
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

    public TypedActionGoalHandle<Feedback, Result> sendGoal(Goal goal) {
        return sendGoal(goal, null, -1.0, null);
    }

    public TypedActionGoalHandle<Feedback, Result> sendGoal(
            Goal goal, Consumer<Feedback> feedback) {
        return sendGoal(goal, null, -1.0, feedback);
    }

    public TypedActionGoalHandle<Feedback, Result> sendGoal(Goal goal, String goalId) {
        return sendGoal(goal, goalId, -1.0, null);
    }

    public TypedActionGoalHandle<Feedback, Result> sendGoal(
            Goal goal, String goalId, double timeoutSecs) {
        return sendGoal(goal, goalId, timeoutSecs, null);
    }

    public TypedActionGoalHandle<Feedback, Result> sendGoal(
            Goal goal, String goalId, double timeoutSecs, Consumer<Feedback> feedback) {
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
        Consumer<ActionMessage> rawFeedback =
                feedback == null
                        ? null
                        : message -> {
                            Feedback decoded = ProtoCodec.tryParse(feedbackType, message.getBody());
                            if (decoded != null) {
                                feedback.accept(decoded);
                            }
                        };
        ActionGoalHandle handle =
                inner.sendGoal(ProtoCodec.encode(goal), goalId, timeoutSecs, rawFeedback);
        return new TypedActionGoalHandle<>(handle, resultType);
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
