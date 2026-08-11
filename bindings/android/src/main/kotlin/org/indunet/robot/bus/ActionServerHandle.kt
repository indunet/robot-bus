package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Opaque action server handle returned by [Node.createActionServer]. */
class ActionServerHandle
internal constructor(
    private val node: Pointer?,
    internal var ptr: Pointer?,
) : AutoCloseable {
    private var destroyed = false

    fun actionName(): String {
        val p = ptr ?: return ""
        return NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_action_server_handle_name(p),
        )
    }

    fun destroy() {
        if (destroyed || ptr == null) return
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_destroy_action_server(node, ptr),
            "destroy_action_server",
        )
        destroyed = true
    }

    override fun close() {
        destroy()
        ptr?.let { RobotBusC.Holder.INSTANCE.robot_bus_action_server_handle_free(it) }
        ptr = null
    }
}
