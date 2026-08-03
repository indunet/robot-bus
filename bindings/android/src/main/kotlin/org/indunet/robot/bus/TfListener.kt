package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Subscribes `/tf` + `/tf_static` (or custom topics) into a shared buffer. */
class TfListener : AutoCloseable {
    private var ptr: Pointer?

    constructor(node: Node) {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_with_defaults(node.raw()),
                "TfListener",
            )
    }

    constructor(node: Node, tfTopic: String, tfStaticTopic: String) {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_new(
                    node.raw(),
                    tfTopic,
                    tfStaticTopic,
                ),
                "TfListener",
            )
    }

    /** Shared buffer handle (Arc clone). Safe to close independently of this listener. */
    fun buffer(): TfBuffer =
        TfBuffer(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_buffer(ptr),
                "TfListener.buffer",
            ),
        )

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_tf_listener_free(ptr)
        ptr = null
    }
}
