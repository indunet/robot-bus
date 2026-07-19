package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Single-threaded executor that can own and spin one or more nodes. */
class SingleThreadedExecutor : AutoCloseable {
    private var ptr: Pointer?

    constructor() {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_new(),
                "SingleThreadedExecutor",
            )
    }

    constructor(context: Context) {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_new_with_context(
                    context.raw(),
                ),
                "SingleThreadedExecutor",
            )
    }

    fun addNode(node: Node) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_add_node(ptr, node.raw()),
            "add_node",
        )
    }

    @JvmOverloads
    fun createNode(name: String, options: NodeOptions? = null): Node =
        Node.fromRaw(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_create_node(
                    ptr,
                    name,
                    options?.toNative(),
                ),
                "create_node",
            ),
        )

    fun shutdownHandle(): ShutdownHandle =
        ShutdownHandle(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_shutdown_handle(ptr),
                "shutdown_handle",
            ),
        )

    fun shutdown() {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_shutdown(ptr),
            "shutdown",
        )
    }

    @JvmOverloads
    fun spinOnce(timeoutSecs: Double = -1.0): Boolean {
        val rc =
            RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_spin_once(ptr, timeoutSecs)
        if (rc < 0) {
            Errors.check(rc, "spin_once")
        }
        return rc == 1
    }

    fun spin() {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_spin(ptr),
            "spin",
        )
    }

    fun start() {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_start(ptr),
            "start",
        )
    }

    fun stop() {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_stop(ptr),
            "stop",
        )
    }

    fun waitForShutdown() {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_wait(ptr),
            "wait",
        )
    }

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_free(ptr)
        ptr = null
    }
}
