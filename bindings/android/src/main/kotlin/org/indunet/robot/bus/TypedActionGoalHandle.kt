package org.indunet.robot.bus

import com.google.protobuf.MessageLite
import java.util.concurrent.CompletableFuture

/** Typed view of an in-flight action goal. */
class TypedActionGoalHandle<Feedback : MessageLite, Result : MessageLite> internal constructor(
    private val inner: ActionGoalHandle,
    private val resultType: Class<Result>,
) : AutoCloseable {
    fun goalId(): String = inner.goalId()

    fun actionName(): String = inner.actionName()

    @JvmOverloads
    fun result(timeoutSecs: Double = -1.0): Result =
        ProtoCodec.parse(resultType, inner.result(timeoutSecs).body)

    @JvmOverloads
    fun resultAsync(timeoutSecs: Double = -1.0): CompletableFuture<Result> =
        inner.resultAsync(timeoutSecs).thenApply { ProtoCodec.parse(resultType, it.body) }

    fun cancel() = inner.cancel()

    override fun close() = inner.close()
}
