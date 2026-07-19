package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Groups callbacks for concurrency control. */
class CallbackGroup internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun id(): Long = RobotBusC.Holder.INSTANCE.robot_bus_callback_group_id(ptr)

    fun kind(): CallbackGroupType {
        val code = RobotBusC.Holder.INSTANCE.robot_bus_callback_group_kind(ptr)
        return if (code == 1) CallbackGroupType.Reentrant else CallbackGroupType.MutuallyExclusive
    }

    internal fun raw(): Pointer? = ptr

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_callback_group_free(ptr)
        ptr = null
    }
}
