package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;
import java.util.concurrent.CompletableFuture;

/** Typed view of an in-flight action goal. */
public final class TypedActionGoalHandle<
                Feedback extends MessageLite, Result extends MessageLite>
        implements AutoCloseable {
    private final ActionGoalHandle inner;
    private final Class<Result> resultType;

    TypedActionGoalHandle(ActionGoalHandle inner, Class<Result> resultType) {
        this.inner = inner;
        this.resultType = resultType;
    }

    public String goalId() {
        return inner.goalId();
    }

    public String actionName() {
        return inner.actionName();
    }

    public Result result() {
        return decode(inner.result());
    }

    public Result result(double timeoutSecs) {
        return decode(inner.result(timeoutSecs));
    }

    public CompletableFuture<Result> resultAsync() {
        return inner.resultAsync().thenApply(this::decode);
    }

    public CompletableFuture<Result> resultAsync(double timeoutSecs) {
        return inner.resultAsync(timeoutSecs).thenApply(this::decode);
    }

    public void cancel() {
        inner.cancel();
    }

    @Override
    public void close() {
        inner.close();
    }

    private Result decode(ActionMessage message) {
        return ProtoCodec.parse(resultType, message.getBody());
    }
}
