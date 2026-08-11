package org.indunet.robot.bus

import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference

/** Client for a named action on a [Node]. */
class ActionClient internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun actionName(): String =
        NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_action_client_action_name(ptr),
        )

    fun actionServerIsReady(): Boolean =
        RobotBusC.Holder.INSTANCE.robot_bus_action_client_action_server_is_ready(ptr) != 0

    @JvmOverloads
    fun waitForActionServer(timeoutSecs: Double = -1.0): Boolean =
        RobotBusC.Holder.INSTANCE.robot_bus_action_client_wait_for_action_server(ptr, timeoutSecs) != 0

    fun sendGoal(body: ByteArray): ActionGoalHandle = sendGoal(body, null, -1.0, null)

    fun sendGoal(
        body: ByteArray,
        feedback: (ActionMessage) -> Unit,
    ): ActionGoalHandle = sendGoal(body, null, -1.0, feedback)

    fun sendGoal(
        body: ByteArray,
        goalId: String?,
    ): ActionGoalHandle = sendGoal(body, goalId, -1.0, null)

    fun sendGoal(
        body: ByteArray,
        goalId: String?,
        timeoutSecs: Double,
    ): ActionGoalHandle = sendGoal(body, goalId, timeoutSecs, null)

    fun sendGoal(
        body: ByteArray,
        goalId: String?,
        timeoutSecs: Double,
        feedback: ((ActionMessage) -> Unit)?,
    ): ActionGoalHandle {
        val callback = ActionGoalHandle.callback(feedback)
        val outHandle = PointerByReference()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_action_client_send_goal(
                ptr,
                body,
                body.size.toLong(),
                goalId,
                timeoutSecs,
                callback,
                null,
                outHandle,
            ),
            "send_goal",
        )
        return ActionGoalHandle(
            Errors.checkPtr(outHandle.value, "send_goal handle"),
            callback,
        )
    }

    override fun close() {
        ptr?.let(RobotBusC.Holder.INSTANCE::robot_bus_action_client_free)
        ptr = null
    }
}
