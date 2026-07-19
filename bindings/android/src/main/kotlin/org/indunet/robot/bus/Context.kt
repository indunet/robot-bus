package org.indunet.robot.bus

import com.sun.jna.Pointer

/**
 * Shared ZeroMQ runtime context. Same-process inproc between an embedded [Broker] and
 * [Node]s requires one shared `Context`.
 */
class Context : AutoCloseable {
    private var ptr: Pointer?

    constructor() {
        ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_context_new(), "Context")
    }

    private constructor(ptr: Pointer) {
        this.ptr = ptr
    }

    /** Cheap clone (refcounted ZMQ context). */
    fun cloneContext(): Context =
        Context(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_context_clone(ptr),
                "Context.clone",
            ),
        )

    internal fun raw(): Pointer? = ptr

    override fun close() {
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_context_free(ptr)
            ptr = null
        }
    }
}
