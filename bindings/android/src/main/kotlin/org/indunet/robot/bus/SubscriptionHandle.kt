package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Opaque subscription handle returned by [Node.createSubscription]. */
class SubscriptionHandle
internal constructor(
    private val node: Pointer?,
    internal var ptr: Pointer?,
) : AutoCloseable {
    private var destroyed = false

    fun destroy() {
        if (destroyed || ptr == null) return
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_destroy_subscription(node, ptr),
            "destroy_subscription",
        )
        destroyed = true
    }

    override fun close() {
        destroy()
        ptr?.let { RobotBusC.Holder.INSTANCE.robot_bus_subscription_handle_free(it) }
        ptr = null
    }
}
