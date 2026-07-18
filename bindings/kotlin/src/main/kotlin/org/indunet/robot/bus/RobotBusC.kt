package org.indunet.robot.bus

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Pointer
import com.sun.jna.Structure
import com.sun.jna.ptr.LongByReference
import com.sun.jna.ptr.PointerByReference

/** JNA mapping of `bindings/cpp/include/robot_bus.h`. */
@Suppress("FunctionName")
internal interface RobotBusC : Library {
    fun robot_bus_last_error(): String?

    fun robot_bus_free_string(s: Pointer?)

    fun robot_bus_free_bytes(data: Pointer?, len: Long)

    fun robot_bus_alloc_bytes(len: Long): Pointer?

    fun robot_bus_dup_string(s: String?): Pointer?

    fun robot_bus_alloc_action_phases(count: Long): Pointer?

    fun robot_bus_action_messages_free(msgs: Pointer?, count: Long)

    fun robot_bus_action_phases_free(phases: Pointer?, count: Long)

    fun robot_bus_message_xsub_endpoint(host: String?, transport: String?, out: PointerByReference): Int

    fun robot_bus_message_xpub_endpoint(host: String?, transport: String?, out: PointerByReference): Int

    fun robot_bus_publisher_new(endpoint: String?): Pointer?

    fun robot_bus_publisher_free(p: Pointer?)

    fun robot_bus_publisher_publish(p: Pointer?, topic: String?, data: ByteArray?, len: Long): Int

    fun robot_bus_publisher_endpoint(p: Pointer?): Pointer?

    fun robot_bus_subscriber_new(endpoint: String?): Pointer?

    fun robot_bus_subscriber_free(s: Pointer?)

    fun robot_bus_subscriber_subscribe(s: Pointer?, topic: String?): Int

    fun robot_bus_subscriber_unsubscribe(s: Pointer?, topic: String?): Int

    fun robot_bus_subscriber_receive(
        s: Pointer?,
        timeoutSecs: Double,
        outTopic: PointerByReference,
        outData: PointerByReference,
        outLen: LongByReference,
    ): Int

    fun robot_bus_subscriber_endpoint(s: Pointer?): Pointer?

    fun robot_bus_shutdown_handle_free(h: Pointer?)

    fun robot_bus_shutdown_handle_shutdown(h: Pointer?)

    fun robot_bus_shutdown_handle_is_running(h: Pointer?): Int

    fun robot_bus_timer_handle_free(h: Pointer?)

    fun robot_bus_callback_group_free(g: Pointer?)

    fun robot_bus_callback_group_id(g: Pointer?): Long

    fun robot_bus_callback_group_kind(g: Pointer?): Int

    fun robot_bus_topic_publisher_free(p: Pointer?)

    fun robot_bus_topic_publisher_topic(p: Pointer?): Pointer?

    fun robot_bus_topic_publisher_publish(p: Pointer?, data: ByteArray?, len: Long): Int

    fun robot_bus_service_client_free(c: Pointer?)

    fun robot_bus_service_client_service_name(c: Pointer?): Pointer?

    fun robot_bus_service_client_call(
        c: Pointer?,
        data: ByteArray?,
        len: Long,
        timeoutSecs: Double,
        outData: PointerByReference,
        outLen: LongByReference,
    ): Int

    fun robot_bus_action_client_free(c: Pointer?)

    fun robot_bus_action_client_action_name(c: Pointer?): Pointer?

    fun robot_bus_action_client_send_goal(
        c: Pointer?,
        data: ByteArray?,
        len: Long,
        goalId: String?,
        timeoutSecs: Double,
        outMsgs: PointerByReference,
        outCount: LongByReference,
    ): Int

    fun robot_bus_action_client_cancel(
        c: Pointer?,
        goalId: String?,
        data: ByteArray?,
        len: Long,
        timeoutSecs: Double,
        outMsg: ActionMessageStruct,
    ): Int

    fun robot_bus_node_new(name: String?, opts: NodeOptions?): Pointer?

    fun robot_bus_node_tcp(name: String?, host: String?): Pointer?

    fun robot_bus_node_ipc(name: String?, path: String?): Pointer?

    fun robot_bus_node_inproc(name: String?, prefix: String?): Pointer?

    fun robot_bus_node_grpc(name: String?): Pointer?

    fun robot_bus_node_grpc_at(name: String?, url: String?): Pointer?

    fun robot_bus_node_free(n: Pointer?)

    fun robot_bus_node_name(n: Pointer?): Pointer?

    fun robot_bus_node_create_callback_group(n: Pointer?, kind: Int): Pointer?

    fun robot_bus_node_create_publisher(n: Pointer?, topic: String?): Pointer?

    fun robot_bus_node_create_subscription(
        n: Pointer?,
        topic: String?,
        callback: MsgCallback,
        user: Pointer?,
        group: Pointer?,
    ): Int

    fun robot_bus_node_create_timer(
        n: Pointer?,
        periodSecs: Double,
        callback: TimerCallback,
        user: Pointer?,
        group: Pointer?,
    ): Pointer?

    fun robot_bus_node_cancel_timer(n: Pointer?, handle: Pointer?): Int

    fun robot_bus_node_create_service(
        n: Pointer?,
        serviceName: String?,
        handler: ServiceHandler,
        user: Pointer?,
        group: Pointer?,
    ): Int

    fun robot_bus_node_create_client(n: Pointer?, serviceName: String?): Pointer?

    fun robot_bus_node_create_action_server(
        n: Pointer?,
        actionName: String?,
        handler: ActionHandler,
        user: Pointer?,
        group: Pointer?,
    ): Int

    fun robot_bus_node_create_action_client(n: Pointer?, actionName: String?): Pointer?

    fun robot_bus_node_connect_action_client(n: Pointer?): Int

    fun robot_bus_node_shutdown_handle(n: Pointer?): Pointer?

    fun robot_bus_node_shutdown(n: Pointer?): Int

    fun robot_bus_node_spin_once(n: Pointer?, timeoutSecs: Double): Int

    fun robot_bus_node_spin(n: Pointer?): Int

    fun robot_bus_node_start(n: Pointer?): Int

    fun robot_bus_node_stop(n: Pointer?): Int

    fun robot_bus_node_wait(n: Pointer?): Int

    fun robot_bus_single_threaded_executor_new(): Pointer?

    fun robot_bus_single_threaded_executor_free(e: Pointer?)

    fun robot_bus_single_threaded_executor_add_node(e: Pointer?, n: Pointer?): Int

    fun robot_bus_single_threaded_executor_create_node(
        e: Pointer?,
        name: String?,
        opts: NodeOptions?,
    ): Pointer?

    fun robot_bus_single_threaded_executor_shutdown_handle(e: Pointer?): Pointer?

    fun robot_bus_single_threaded_executor_shutdown(e: Pointer?): Int

    fun robot_bus_single_threaded_executor_spin_once(e: Pointer?, timeoutSecs: Double): Int

    fun robot_bus_single_threaded_executor_spin(e: Pointer?): Int

    fun robot_bus_single_threaded_executor_start(e: Pointer?): Int

    fun robot_bus_single_threaded_executor_stop(e: Pointer?): Int

    fun robot_bus_single_threaded_executor_wait(e: Pointer?): Int

    fun robot_bus_multi_threaded_executor_new(numThreads: Long): Pointer?

    fun robot_bus_multi_threaded_executor_free(e: Pointer?)

    fun robot_bus_multi_threaded_executor_add_node(e: Pointer?, n: Pointer?): Int

    fun robot_bus_multi_threaded_executor_create_node(
        e: Pointer?,
        name: String?,
        opts: NodeOptions?,
    ): Pointer?

    fun robot_bus_multi_threaded_executor_shutdown_handle(e: Pointer?): Pointer?

    fun robot_bus_multi_threaded_executor_shutdown(e: Pointer?): Int

    fun robot_bus_multi_threaded_executor_spin_once(e: Pointer?, timeoutSecs: Double): Int

    fun robot_bus_multi_threaded_executor_spin(e: Pointer?): Int

    fun robot_bus_broker_start(opts: BrokerOptions?): Pointer?

    fun robot_bus_broker_free(b: Pointer?)

    fun robot_bus_broker_stop(b: Pointer?): Int

    fun robot_bus_broker_message_xsub_bind(b: Pointer?): Pointer?

    fun robot_bus_broker_message_xpub_bind(b: Pointer?): Pointer?

    fun robot_bus_broker_service_frontend_bind(b: Pointer?): Pointer?

    fun robot_bus_broker_service_backend_bind(b: Pointer?): Pointer?

    fun robot_bus_broker_action_frontend_bind(b: Pointer?): Pointer?

    fun robot_bus_broker_action_backend_bind(b: Pointer?): Pointer?

    fun robot_bus_broker_grpc_listen(b: Pointer?): Pointer?

    fun robot_bus_broker_console_listen(b: Pointer?): Pointer?

    fun interface MsgCallback : Callback {
        fun invoke(topic: String?, data: Pointer?, len: Long, user: Pointer?)
    }

    fun interface TimerCallback : Callback {
        fun invoke(user: Pointer?)
    }

    fun interface ServiceHandler : Callback {
        fun invoke(
            data: Pointer?,
            len: Long,
            outData: PointerByReference,
            outLen: LongByReference,
            user: Pointer?,
        ): Int
    }

    fun interface ActionHandler : Callback {
        fun invoke(
            data: Pointer?,
            len: Long,
            outPhases: PointerByReference,
            outCount: LongByReference,
            user: Pointer?,
        ): Int
    }

    @Structure.FieldOrder(
        "host",
        "transport",
        "grpcUrl",
        "messageXsub",
        "messageXpub",
        "serviceFrontend",
        "serviceBackend",
        "actionBackend",
        "actionFrontend",
    )
    open class NodeOptions : Structure() {
        @JvmField var host: String? = null
        @JvmField var transport: String? = null
        @JvmField var grpcUrl: String? = null
        @JvmField var messageXsub: String? = null
        @JvmField var messageXpub: String? = null
        @JvmField var serviceFrontend: String? = null
        @JvmField var serviceBackend: String? = null
        @JvmField var actionBackend: String? = null
        @JvmField var actionFrontend: String? = null
    }

    @Structure.FieldOrder(
        "messageXsubBind",
        "messageXpubBind",
        "serviceFrontendBind",
        "serviceBackendBind",
        "actionFrontendBind",
        "actionBackendBind",
        "grpcListen",
        "consoleListen",
        "tcpOnly",
        "noConsole",
    )
    open class BrokerOptions : Structure() {
        @JvmField var messageXsubBind: String? = null
        @JvmField var messageXpubBind: String? = null
        @JvmField var serviceFrontendBind: String? = null
        @JvmField var serviceBackendBind: String? = null
        @JvmField var actionFrontendBind: String? = null
        @JvmField var actionBackendBind: String? = null
        @JvmField var grpcListen: String? = null
        @JvmField var consoleListen: String? = null
        @JvmField var tcpOnly: Int = 0
        @JvmField var noConsole: Int = 0
    }

    @Structure.FieldOrder("kind", "body", "bodyLen", "goalId", "actionName")
    open class ActionMessageStruct : Structure {
        @JvmField var kind: Pointer? = null
        @JvmField var body: Pointer? = null
        @JvmField var bodyLen: Long = 0
        @JvmField var goalId: Pointer? = null
        @JvmField var actionName: Pointer? = null

        constructor() : super()
        constructor(p: Pointer) : super(p) {
            read()
        }
    }

    @Structure.FieldOrder("phase", "body", "bodyLen")
    open class ActionPhaseStruct : Structure {
        @JvmField var phase: Pointer? = null
        @JvmField var body: Pointer? = null
        @JvmField var bodyLen: Long = 0

        constructor() : super()
        constructor(p: Pointer) : super(p) {
            read()
        }
    }

    companion object {
        val INSTANCE: RobotBusC by lazy { NativeLoader.loadLibrary() }
    }
}
