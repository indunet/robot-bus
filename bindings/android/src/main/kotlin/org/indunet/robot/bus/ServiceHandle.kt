package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Opaque service server handle returned by [Node.createService]. */
class ServiceHandle
internal constructor(
    private val node: Pointer?,
    internal var ptr: Pointer?,
) : AutoCloseable {
    private var destroyed = false

    fun serviceName(): String {
        val p = ptr ?: return ""
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_service_handle_name(p))
    }

    fun destroy() {
        if (destroyed || ptr == null) return
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_destroy_service(node, ptr),
            "destroy_service",
        )
        destroyed = true
    }

    override fun close() {
        destroy()
        ptr?.let { RobotBusC.Holder.INSTANCE.robot_bus_service_handle_free(it) }
        ptr = null
    }
}
