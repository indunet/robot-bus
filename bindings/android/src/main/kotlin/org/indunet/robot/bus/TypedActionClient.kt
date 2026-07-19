package org.indunet.robot.bus

import com.google.protobuf.MessageLite

/** Action client that encodes goals and decodes FEEDBACK / RESULT bodies. */
class TypedActionClient<
    Goal : MessageLite,
    Feedback : MessageLite,
    Result : MessageLite,
> internal constructor(
    private val inner: ActionClient,
    private val goalType: Class<Goal>,
    private val feedbackType: Class<Feedback>,
    private val resultType: Class<Result>,
) : AutoCloseable {
    fun actionName(): String = inner.actionName()

    fun goalType(): Class<Goal> = goalType

    fun feedbackType(): Class<Feedback> = feedbackType

    fun resultType(): Class<Result> = resultType

    @JvmOverloads
    fun sendGoal(
        goal: Goal,
        goalId: String? = null,
        timeoutSecs: Double = -1.0,
    ): List<TypedActionMessage> {
        requireNotNull(goal) { "goal" }
        if (!goalType.isInstance(goal)) {
            throw IllegalArgumentException(
                "action client for ${goalType.simpleName} got ${goal.javaClass.simpleName}",
            )
        }
        val raw = inner.sendGoal(ProtoCodec.encode(goal), goalId, timeoutSecs)
        val out = ArrayList<TypedActionMessage>(raw.size)
        for (msg in raw) {
            val decoded = decode(msg)
            if (decoded.body == null && isTypedKind(msg.kind) && msg.body.isNotEmpty()) {
                continue
            }
            out.add(decoded)
        }
        return out
    }

    @JvmOverloads
    fun cancel(
        goalId: String,
        body: MessageLite? = null,
        timeoutSecs: Double = -1.0,
    ): TypedActionMessage {
        val raw = if (body == null) ByteArray(0) else ProtoCodec.encode(body)
        return decode(inner.cancel(goalId, raw, timeoutSecs))
    }

    private fun decode(msg: ActionMessage): TypedActionMessage {
        val kind = msg.kind
        val decoded: MessageLite? =
            when {
                kind.equals("FEEDBACK", ignoreCase = true) ->
                    ProtoCodec.tryParse(feedbackType, msg.body)
                kind.equals("RESULT", ignoreCase = true) ->
                    ProtoCodec.tryParse(resultType, msg.body)
                kind.equals("GOAL", ignoreCase = true) ->
                    ProtoCodec.tryParse(goalType, msg.body)
                else -> null
            }
        return TypedActionMessage(kind, decoded, msg.body, msg.goalId, msg.actionName)
    }

    private fun isTypedKind(kind: String): Boolean =
        kind.equals("FEEDBACK", ignoreCase = true) ||
            kind.equals("RESULT", ignoreCase = true) ||
            kind.equals("GOAL", ignoreCase = true)

    override fun close() {
        inner.close()
    }

    override fun toString(): String =
        "TypedActionClient{action=${actionName()}, goal=${goalType.simpleName}}"
}
