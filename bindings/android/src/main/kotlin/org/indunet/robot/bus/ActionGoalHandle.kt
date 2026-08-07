package org.indunet.robot.bus

import com.sun.jna.Pointer
import java.util.concurrent.CompletableFuture
import java.util.concurrent.locks.ReentrantReadWriteLock

/** Owns one in-flight action goal and its native feedback callback. */
class ActionGoalHandle internal constructor(
    private var ptr: Pointer?,
    // Keep the JNA callback strongly reachable until the native handle is freed.
    private val feedbackCallback: RobotBusC.ActionFeedbackCb?,
) : AutoCloseable {
    private val lifecycle = ReentrantReadWriteLock()

    fun goalId(): String =
        withReadLock {
            NativeUtils.takeCString(
                RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_goal_id(requireOpen()),
            )
        }

    fun actionName(): String =
        withReadLock {
            NativeUtils.takeCString(
                RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_action_name(requireOpen()),
            )
        }

    @JvmOverloads
    fun result(timeoutSecs: Double = -1.0): ActionMessage =
        withReadLock {
            val out = RobotBusC.ActionMessageStruct()
            Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_wait_result(
                    requireOpen(),
                    timeoutSecs,
                    out,
                ),
                "action_goal_result",
            )
            out.read()
            try {
                copyMessage(out)
            } finally {
                freeMessage(out)
            }
        }

    @JvmOverloads
    fun resultAsync(timeoutSecs: Double = -1.0): CompletableFuture<ActionMessage> =
        CompletableFuture.supplyAsync { result(timeoutSecs) }

    fun cancel() {
        withReadLock {
            Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_action_goal_handle_cancel(requireOpen()),
                "action_goal_cancel",
            )
        }
    }

    override fun close() {
        lifecycle.writeLock().lock()
        try {
            ptr?.let(RobotBusC.Holder.INSTANCE::robot_bus_action_goal_handle_free)
            ptr = null
        } finally {
            lifecycle.writeLock().unlock()
        }
    }

    private inline fun <T> withReadLock(block: () -> T): T {
        lifecycle.readLock().lock()
        return try {
            block()
        } finally {
            lifecycle.readLock().unlock()
        }
    }

    private fun requireOpen(): Pointer =
        ptr ?: throw IllegalStateException("action goal handle is closed")

    internal companion object {
        fun callback(feedback: ((ActionMessage) -> Unit)?): RobotBusC.ActionFeedbackCb? =
            feedback?.let { consumer ->
                RobotBusC.ActionFeedbackCb { message, _ ->
                    if (message != null) {
                        consumer(copyMessage(RobotBusC.ActionMessageStruct(message)))
                    }
                }
            }

        private fun copyMessage(msg: RobotBusC.ActionMessageStruct): ActionMessage {
            val body =
                if (msg.body != null && msg.bodyLen > 0) {
                    msg.body!!.getByteArray(0, msg.bodyLen.toInt())
                } else {
                    ByteArray(0)
                }
            return ActionMessage(
                msg.kind?.getString(0) ?: "",
                body,
                msg.goalId?.getString(0) ?: "",
                msg.actionName?.getString(0) ?: "",
            )
        }

        private fun freeMessage(msg: RobotBusC.ActionMessageStruct) {
            RobotBusC.Holder.INSTANCE.robot_bus_free_string(msg.kind)
            RobotBusC.Holder.INSTANCE.robot_bus_free_bytes(msg.body, msg.bodyLen)
            RobotBusC.Holder.INSTANCE.robot_bus_free_string(msg.goalId)
            RobotBusC.Holder.INSTANCE.robot_bus_free_string(msg.actionName)
        }
    }
}
