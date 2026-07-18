package org.indunet.robot.bus

import com.sun.jna.Pointer

class Broker private constructor(private var ptr: Pointer?) : AutoCloseable {
    constructor() : this(checkPtr(RobotBusC.INSTANCE.robot_bus_broker_start(null), "Broker"))

    constructor(options: BrokerOptions) : this(
        checkPtr(RobotBusC.INSTANCE.robot_bus_broker_start(options.toNative()), "Broker"),
    )

    fun stop() {
        check(RobotBusC.INSTANCE.robot_bus_broker_stop(ptr), "broker stop")
    }

    fun messageXsubBind(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_message_xsub_bind(ptr))

    fun messageXpubBind(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_message_xpub_bind(ptr))

    fun serviceFrontendBind(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_service_frontend_bind(ptr))

    fun serviceBackendBind(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_service_backend_bind(ptr))

    fun actionFrontendBind(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_action_frontend_bind(ptr))

    fun actionBackendBind(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_action_backend_bind(ptr))

    fun grpcListen(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_grpc_listen(ptr))

    fun consoleListen(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_broker_console_listen(ptr))

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_broker_free(ptr)
        ptr = null
    }
}

class SingleThreadedExecutor : AutoCloseable {
    private var ptr: Pointer? =
        checkPtr(RobotBusC.INSTANCE.robot_bus_single_threaded_executor_new(), "SingleThreadedExecutor")

    fun addNode(node: Node) {
        check(
            RobotBusC.INSTANCE.robot_bus_single_threaded_executor_add_node(ptr, node.raw()),
            "add_node",
        )
    }

    fun createNode(name: String, options: NodeOptions? = null): Node =
        Node.fromRaw(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_single_threaded_executor_create_node(
                    ptr,
                    name,
                    options?.toNative(),
                ),
                "create_node",
            ),
        )

    fun shutdownHandle(): ShutdownHandle =
        ShutdownHandle(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_single_threaded_executor_shutdown_handle(ptr),
                "shutdown_handle",
            ),
        )

    fun shutdown() {
        check(RobotBusC.INSTANCE.robot_bus_single_threaded_executor_shutdown(ptr), "shutdown")
    }

    fun spinOnce(timeoutSecs: Double = -1.0): Boolean {
        val rc = RobotBusC.INSTANCE.robot_bus_single_threaded_executor_spin_once(ptr, timeoutSecs)
        if (rc < 0) check(rc, "spin_once")
        return rc == 1
    }

    fun spin() {
        check(RobotBusC.INSTANCE.robot_bus_single_threaded_executor_spin(ptr), "spin")
    }

    fun start() {
        check(RobotBusC.INSTANCE.robot_bus_single_threaded_executor_start(ptr), "start")
    }

    fun stop() {
        check(RobotBusC.INSTANCE.robot_bus_single_threaded_executor_stop(ptr), "stop")
    }

    fun waitForShutdown() {
        check(RobotBusC.INSTANCE.robot_bus_single_threaded_executor_wait(ptr), "wait")
    }

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_single_threaded_executor_free(ptr)
        ptr = null
    }
}

class MultiThreadedExecutor(numThreads: Int = 0) : AutoCloseable {
    private var ptr: Pointer? =
        checkPtr(
            RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_new(numThreads.toLong()),
            "MultiThreadedExecutor",
        )

    fun addNode(node: Node) {
        check(
            RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_add_node(ptr, node.raw()),
            "add_node",
        )
    }

    fun createNode(name: String, options: NodeOptions? = null): Node =
        Node.fromRaw(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_create_node(
                    ptr,
                    name,
                    options?.toNative(),
                ),
                "create_node",
            ),
        )

    fun shutdownHandle(): ShutdownHandle =
        ShutdownHandle(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_shutdown_handle(ptr),
                "shutdown_handle",
            ),
        )

    fun shutdown() {
        check(RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_shutdown(ptr), "shutdown")
    }

    fun spinOnce(timeoutSecs: Double = -1.0): Boolean {
        val rc = RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_spin_once(ptr, timeoutSecs)
        if (rc < 0) check(rc, "spin_once")
        return rc == 1
    }

    fun spin() {
        check(RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_spin(ptr), "spin")
    }

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_multi_threaded_executor_free(ptr)
        ptr = null
    }
}
