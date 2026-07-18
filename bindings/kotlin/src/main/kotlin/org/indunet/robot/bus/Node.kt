package org.indunet.robot.bus

import com.sun.jna.Pointer
import java.util.concurrent.CopyOnWriteArrayList

class Node private constructor(private var ptr: Pointer?) : AutoCloseable {
    private val msgCallbacks = CopyOnWriteArrayList<RobotBusC.MsgCallback>()
    private val timerCallbacks = CopyOnWriteArrayList<RobotBusC.TimerCallback>()
    private val serviceHandlers = CopyOnWriteArrayList<RobotBusC.ServiceHandler>()
    private val actionHandlers = CopyOnWriteArrayList<RobotBusC.ActionHandler>()

    constructor(name: String, options: NodeOptions? = null) : this(
        checkPtr(RobotBusC.INSTANCE.robot_bus_node_new(name, options?.toNative()), "Node"),
    )

    companion object {
        fun tcp(name: String, host: String = "localhost"): Node =
            Node(checkPtr(RobotBusC.INSTANCE.robot_bus_node_tcp(name, host), "Node.tcp"))

        fun ipc(name: String, path: String? = null): Node =
            Node(checkPtr(RobotBusC.INSTANCE.robot_bus_node_ipc(name, path), "Node.ipc"))

        fun inproc(name: String, prefix: String? = null): Node =
            Node(checkPtr(RobotBusC.INSTANCE.robot_bus_node_inproc(name, prefix), "Node.inproc"))

        fun grpc(name: String): Node =
            Node(checkPtr(RobotBusC.INSTANCE.robot_bus_node_grpc(name), "Node.grpc"))

        fun grpcAt(name: String, url: String): Node =
            Node(checkPtr(RobotBusC.INSTANCE.robot_bus_node_grpc_at(name, url), "Node.grpcAt"))

        internal fun fromRaw(ptr: Pointer): Node = Node(ptr)
    }

    fun name(): String = takeCString(RobotBusC.INSTANCE.robot_bus_node_name(ptr))

    fun createCallbackGroup(kind: CallbackGroupType = CallbackGroupType.MutuallyExclusive): CallbackGroup =
        CallbackGroup(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_node_create_callback_group(ptr, kind.code),
                "create_callback_group",
            ),
        )

    fun createPublisher(topic: String): TopicPublisher =
        TopicPublisher(
            checkPtr(RobotBusC.INSTANCE.robot_bus_node_create_publisher(ptr, topic), "create_publisher"),
        )

    fun createSubscription(
        topic: String,
        callback: (topic: String, payload: ByteArray) -> Unit,
        group: CallbackGroup? = null,
    ) {
        val cb = RobotBusC.MsgCallback { t, data, len, _ ->
            val bytes =
                if (data != null && len > 0) data.getByteArray(0, len.toInt()) else ByteArray(0)
            callback(t.orEmpty(), bytes)
        }
        msgCallbacks.add(cb)
        check(
            RobotBusC.INSTANCE.robot_bus_node_create_subscription(
                ptr,
                topic,
                cb,
                null,
                group?.raw(),
            ),
            "create_subscription",
        )
    }

    fun createTimer(
        periodSecs: Double,
        callback: () -> Unit,
        group: CallbackGroup? = null,
    ): TimerHandle {
        val cb = RobotBusC.TimerCallback { _ -> callback() }
        timerCallbacks.add(cb)
        return TimerHandle(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_node_create_timer(
                    ptr,
                    periodSecs,
                    cb,
                    null,
                    group?.raw(),
                ),
                "create_timer",
            ),
        )
    }

    fun cancelTimer(handle: TimerHandle) {
        check(RobotBusC.INSTANCE.robot_bus_node_cancel_timer(ptr, handle.ptr), "cancel_timer")
    }

    fun createService(
        serviceName: String,
        handler: (body: ByteArray) -> ByteArray,
        group: CallbackGroup? = null,
    ) {
        val cb = RobotBusC.ServiceHandler { data, len, outData, outLen, _ ->
            try {
                val body =
                    if (data != null && len > 0) data.getByteArray(0, len.toInt()) else ByteArray(0)
                val reply = handler(body)
                outLen.value = reply.size.toLong()
                outData.value = allocReplyBytes(reply)
                0
            } catch (_: Exception) {
                outData.value = null
                outLen.value = 0
                -1
            }
        }
        serviceHandlers.add(cb)
        check(
            RobotBusC.INSTANCE.robot_bus_node_create_service(
                ptr,
                serviceName,
                cb,
                null,
                group?.raw(),
            ),
            "create_service",
        )
    }

    fun createClient(serviceName: String): ServiceClient =
        ServiceClient(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_node_create_client(ptr, serviceName),
                "create_client",
            ),
        )

    fun createActionServer(
        actionName: String,
        handler: (body: ByteArray) -> List<ActionPhase>,
        group: CallbackGroup? = null,
    ) {
        val cb = RobotBusC.ActionHandler { data, len, outPhases, outCount, _ ->
            try {
                val body =
                    if (data != null && len > 0) data.getByteArray(0, len.toInt()) else ByteArray(0)
                val phases = handler(body)
                outCount.value = phases.size.toLong()
                if (phases.isEmpty()) {
                    outPhases.value = null
                    return@ActionHandler 0
                }
                val arr = RobotBusC.INSTANCE.robot_bus_alloc_action_phases(phases.size.toLong())
                if (arr == null) {
                    outPhases.value = null
                    outCount.value = 0
                    return@ActionHandler -1
                }
                val stride = RobotBusC.ActionPhaseStruct().size().toLong()
                phases.forEachIndexed { i, phase ->
                    val item = RobotBusC.ActionPhaseStruct(arr.share(i * stride))
                    item.phase = RobotBusC.INSTANCE.robot_bus_dup_string(phase.phase)
                    item.bodyLen = phase.body.size.toLong()
                    item.body = allocReplyBytes(phase.body)
                    item.write()
                }
                outPhases.value = arr
                0
            } catch (_: Exception) {
                outPhases.value = null
                outCount.value = 0
                -1
            }
        }
        actionHandlers.add(cb)
        check(
            RobotBusC.INSTANCE.robot_bus_node_create_action_server(
                ptr,
                actionName,
                cb,
                null,
                group?.raw(),
            ),
            "create_action_server",
        )
    }

    fun createActionClient(actionName: String): ActionClient =
        ActionClient(
            checkPtr(
                RobotBusC.INSTANCE.robot_bus_node_create_action_client(ptr, actionName),
                "create_action_client",
            ),
        )

    fun connectActionClient() {
        check(RobotBusC.INSTANCE.robot_bus_node_connect_action_client(ptr), "connect_action_client")
    }

    fun shutdownHandle(): ShutdownHandle =
        ShutdownHandle(
            checkPtr(RobotBusC.INSTANCE.robot_bus_node_shutdown_handle(ptr), "shutdown_handle"),
        )

    fun shutdown() {
        check(RobotBusC.INSTANCE.robot_bus_node_shutdown(ptr), "shutdown")
    }

    /** @return true if work was processed */
    fun spinOnce(timeoutSecs: Double = -1.0): Boolean {
        val rc = RobotBusC.INSTANCE.robot_bus_node_spin_once(ptr, timeoutSecs)
        if (rc < 0) check(rc, "spin_once")
        return rc == 1
    }

    fun spin() {
        check(RobotBusC.INSTANCE.robot_bus_node_spin(ptr), "spin")
    }

    fun start() {
        check(RobotBusC.INSTANCE.robot_bus_node_start(ptr), "start")
    }

    fun stop() {
        check(RobotBusC.INSTANCE.robot_bus_node_stop(ptr), "stop")
    }

    fun waitForShutdown() {
        check(RobotBusC.INSTANCE.robot_bus_node_wait(ptr), "wait")
    }

    internal fun raw(): Pointer? = ptr

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_node_free(ptr)
        ptr = null
        msgCallbacks.clear()
        timerCallbacks.clear()
        serviceHandlers.clear()
        actionHandlers.clear()
    }
}
