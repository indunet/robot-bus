package org.indunet.robot.bus

import com.sun.jna.Pointer

/** In-process / local broker process handle. */
class Broker private constructor(private var ptr: Pointer?) : AutoCloseable {
    constructor() : this(start(null, null))

    constructor(options: BrokerOptions) : this(start(null, options))

    /** Start broker sharing [context] (required for same-process inproc Nodes). */
    constructor(context: Context) : this(start(context, null))

    constructor(context: Context, options: BrokerOptions?) : this(start(context, options))

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

    companion object {
        private fun start(context: Context?, options: BrokerOptions?): Pointer {
            val native = RobotBusC.Holder.INSTANCE
            val nativeOpts = options?.toNative()
            val ipcRoot = RobotBusAndroid.ipcRoot
            val broker = if (ipcRoot == null) {
                native.robot_bus_broker_start_with_context(context?.raw(), nativeOpts)
            } else {
                native.robot_bus_broker_start_with_context_in(context?.raw(), nativeOpts, ipcRoot)
            }
            return Errors.checkPtr(broker, "Broker")
        }
    }
}
