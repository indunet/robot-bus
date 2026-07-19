package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Opaque timer handle returned by [Node.createTimer]. */
class TimerHandle internal constructor(internal val ptr: Pointer?) : AutoCloseable {
    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_timer_handle_free(ptr)
    }
}
