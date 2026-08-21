package org.indunet.robot.bus

import com.sun.jna.Pointer

/** In-process / local broker process handle. */
class Broker : AutoCloseable {
    private var ptr: Pointer?

    constructor() {
        ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_broker_start(null), "Broker")
    }

    constructor(options: BrokerOptions) {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_start(options.toNative()),
                "Broker",
            )
    }

    /** Start broker sharing [context] (required for same-process inproc Nodes). */
    constructor(context: Context) {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_start_with_context(context.raw(), null),
                "Broker",
            )
    }

    constructor(context: Context, options: BrokerOptions?) {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_start_with_context(
                    context.raw(),
                    options?.toNative(),
                ),
                "Broker",
            )
    }

    fun stop() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_broker_stop(ptr), "broker stop")
    }

    fun messageXsubBind(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_message_xsub_bind(ptr))

    fun messageXpubBind(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_message_xpub_bind(ptr))

    fun serviceFrontendBind(): String =
        NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_broker_service_frontend_bind(ptr),
        )

    fun serviceBackendBind(): String =
        NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_broker_service_backend_bind(ptr),
        )

    fun actionFrontendBind(): String =
        NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_broker_action_frontend_bind(ptr),
        )

    fun actionBackendBind(): String =
        NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_broker_action_backend_bind(ptr),
        )

    fun apiListen(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_api_listen(ptr))

    fun consoleListen(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_console_listen(ptr))

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_broker_free(ptr)
        ptr = null
    }
}
