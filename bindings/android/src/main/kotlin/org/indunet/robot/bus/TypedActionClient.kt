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

    fun actionServerIsReady(): Boolean = inner.actionServerIsReady()

    @JvmOverloads
    fun waitForActionServer(timeoutSecs: Double = -1.0): Boolean =
        inner.waitForActionServer(timeoutSecs)

    fun goalType(): Class<Goal> = goalType

    fun feedbackType(): Class<Feedback> = feedbackType

    fun resultType(): Class<Result> = resultType

    fun sendGoal(goal: Goal): TypedActionGoalHandle<Feedback, Result> =
        sendGoal(goal, null, -1.0, null)

    fun sendGoal(
        goal: Goal,
        feedback: (Feedback) -> Unit,
    ): TypedActionGoalHandle<Feedback, Result> = sendGoal(goal, null, -1.0, feedback)

    fun sendGoal(
        goal: Goal,
        goalId: String?,
    ): TypedActionGoalHandle<Feedback, Result> = sendGoal(goal, goalId, -1.0, null)

    fun sendGoal(
        goal: Goal,
        goalId: String?,
        timeoutSecs: Double,
    ): TypedActionGoalHandle<Feedback, Result> = sendGoal(goal, goalId, timeoutSecs, null)

    fun sendGoal(
        goal: Goal,
        goalId: String?,
        timeoutSecs: Double,
        feedback: ((Feedback) -> Unit)?,
    ): TypedActionGoalHandle<Feedback, Result> {
        requireNotNull(goal) { "goal" }
        if (!goalType.isInstance(goal)) {
            throw IllegalArgumentException(
                "action client for ${goalType.simpleName} got ${goal.javaClass.simpleName}",
            )
        }
        val rawFeedback: ((ActionMessage) -> Unit)? =
            feedback?.let { consumer ->
                { message ->
                    ProtoCodec.tryParse(feedbackType, message.body)?.let(consumer)
                }
            }
        return TypedActionGoalHandle(
            inner.sendGoal(ProtoCodec.encode(goal), goalId, timeoutSecs, rawFeedback),
            resultType,
        )
    }

    override fun close() {
        inner.close()
    }

    override fun toString(): String =
        "TypedActionClient{action=${actionName()}, goal=${goalType.simpleName}}"
}
