package org.indunet.robot.bus

import com.sun.jna.Pointer
import com.sun.jna.ptr.LongByReference
import com.sun.jna.ptr.PointerByReference

/** Client for a named action on a [Node]. */
class ActionClient internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun actionName(): String =
        NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_action_client_action_name(ptr),
        )

    @JvmOverloads
    fun sendGoal(
        body: ByteArray,
        goalId: String? = null,
        timeoutSecs: Double = -1.0,
    ): List<ActionMessage> {
        val outMsgs = PointerByReference()
        val outCount = LongByReference()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_action_client_send_goal(
                ptr,
                body,
                body.size.toLong(),
                goalId,
                timeoutSecs,
                outMsgs,
                outCount,
            ),
            "send_goal",
        )
        val count = outCount.value
        val base = outMsgs.value
        if (base == null || count <= 0) {
            return emptyList()
        }
        try {
            val size = RobotBusC.ActionMessageStruct().size().toLong()
            val result = ArrayList<ActionMessage>(count.toInt())
            for (i in 0 until count) {
                val msg = RobotBusC.ActionMessageStruct(base.share(i * size))
                val kind = msg.kind?.getString(0) ?: ""
                val bodyBytes =
                    if (msg.body != null && msg.bodyLen > 0) {
                        msg.body!!.getByteArray(0, msg.bodyLen.toInt())
                    } else {
                        ByteArray(0)
                    }
                val gid = msg.goalId?.getString(0) ?: ""
                val name = msg.actionName?.getString(0) ?: ""
                result.add(ActionMessage(kind, bodyBytes, gid, name))
            }
            return result
        } finally {
            RobotBusC.Holder.INSTANCE.robot_bus_action_messages_free(base, count)
        }
    }

    @JvmOverloads
    fun cancel(
        goalId: String,
        body: ByteArray = ByteArray(0),
        timeoutSecs: Double = -1.0,
    ): ActionMessage {
        val out = RobotBusC.ActionMessageStruct()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_action_client_cancel(
                ptr,
                goalId,
                body,
                body.size.toLong(),
                timeoutSecs,
                out,
            ),
            "cancel",
        )
        out.read()
        val kind = out.kind?.getString(0) ?: ""
        val bodyBytes =
            if (out.body != null && out.bodyLen > 0) {
                out.body!!.getByteArray(0, out.bodyLen.toInt())
            } else {
                ByteArray(0)
            }
        val gid = out.goalId?.getString(0) ?: ""
        val name = out.actionName?.getString(0) ?: ""
        val result = ActionMessage(kind, bodyBytes, gid, name)
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(out.kind)
        RobotBusC.Holder.INSTANCE.robot_bus_free_bytes(out.body, out.bodyLen)
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(out.goalId)
        RobotBusC.Holder.INSTANCE.robot_bus_free_string(out.actionName)
        return result
    }

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_action_client_free(ptr)
        ptr = null
    }
}
