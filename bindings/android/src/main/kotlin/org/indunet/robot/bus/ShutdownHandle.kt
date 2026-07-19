package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Handle used to request shutdown from another thread. */
class ShutdownHandle internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun shutdown() {
        RobotBusC.Holder.INSTANCE.robot_bus_shutdown_handle_shutdown(ptr)
    }

    fun isRunning(): Boolean =
        RobotBusC.Holder.INSTANCE.robot_bus_shutdown_handle_is_running(ptr) != 0

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_shutdown_handle_free(ptr)
        ptr = null
    }
}
