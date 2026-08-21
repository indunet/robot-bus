package org.indunet.robot.bus

import com.google.protobuf.MessageLite
import com.sun.jna.Pointer
import com.sun.jna.ptr.LongByReference
import com.sun.jna.ptr.PointerByReference
import java.util.concurrent.CopyOnWriteArrayList

/** Robot-bus node: topics, services, actions, timers, and spinning. */
class Node : AutoCloseable {
    private var ptr: Pointer?
    private val msgCallbacks = CopyOnWriteArrayList<RobotBusC.MsgCb>()
    private val timerCallbacks = CopyOnWriteArrayList<RobotBusC.TimerCb>()
    private val serviceHandlers = CopyOnWriteArrayList<RobotBusC.ServiceCb>()
    private val actionHandlers = CopyOnWriteArrayList<RobotBusC.ActionCb>()

    @JvmOverloads
    constructor(name: String, options: NodeOptions? = null) {
        ptr =
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_new(name, options?.toNative()),
                "Node",
            )
    }

    private constructor(ptr: Pointer) {
        this.ptr = ptr
    }

    fun name(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_node_name(ptr))

    fun connectionState(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_node_connection_state(ptr))

    /** `timeoutSecs < 0` waits until connected or shutdown. */
    @JvmOverloads
    fun waitForBroker(timeoutSecs: Double = -1.0): Boolean =
        RobotBusC.Holder.INSTANCE.robot_bus_node_wait_for_broker(ptr, timeoutSecs) != 0

    @JvmOverloads
    fun createCallbackGroup(
        kind: CallbackGroupType = CallbackGroupType.MutuallyExclusive,
    ): CallbackGroup =
        CallbackGroup(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_callback_group(ptr, kind.code),
                "create_callback_group",
            ),
        )

    @JvmOverloads
    fun createPublisher(topic: String, qosDepth: Int = 0): TopicPublisher =
        TopicPublisher(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_publisher_with_qos(ptr, topic, qosDepth),
                "create_publisher",
            ),
        )

    /** Typed publisher: `publish(Message)` with automatic protobuf encode. */
    @JvmOverloads
    fun <T : MessageLite> createPublisher(
        topic: String,
        msgType: Class<T>,
        qosDepth: Int = 0,
    ): TypedTopicPublisher<T> {
        ProtoCodec.requireMessageType(msgType, "msgType")
        @Suppress("UNCHECKED_CAST")
        val typed = msgType as Class<T>
        return TypedTopicPublisher(createPublisher(topic, qosDepth), typed)
    }

    @JvmOverloads
    fun createSubscription(
        topic: String,
        callback: MsgCallback,
        group: CallbackGroup? = null,
        qosDepth: Int = 0,
    ): SubscriptionHandle {
        val cb =
            RobotBusC.MsgCb { t, data, len, _ ->
                val bytes =
                    if (data != null && len > 0) data.getByteArray(0, len.toInt()) else ByteArray(0)
                callback.onMessage(t ?: "", bytes)
            }
        msgCallbacks.add(cb)
        return SubscriptionHandle(
            ptr,
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_subscription_with_qos(
                    ptr,
                    topic,
                    cb,
                    null,
                    group?.raw(),
                    qosDepth,
                ),
                "create_subscription",
            ),
        )
    }

    /** Typed subscription: callback receives a decoded protobuf message. */
    @JvmOverloads
    fun <T : MessageLite> createSubscription(
        topic: String,
        callback: TypedMsgCallback<T>,
        msgType: Class<T>,
        group: CallbackGroup? = null,
        qosDepth: Int = 0,
    ): SubscriptionHandle {
        ProtoCodec.requireMessageType(msgType, "msgType")
        @Suppress("UNCHECKED_CAST")
        val typed = msgType as Class<T>
        return createSubscription(
            topic,
            { t, payload ->
                val msg = ProtoCodec.tryParse(typed, payload) ?: return@createSubscription
                callback.onMessage(t, msg)
            },
            group,
            qosDepth,
        )
    }

    @JvmOverloads
    fun createTimer(
        periodSecs: Double,
        callback: TimerCallback,
        group: CallbackGroup? = null,
    ): TimerHandle {
        val cb = RobotBusC.TimerCb { _ -> callback.onTimer() }
        timerCallbacks.add(cb)
        return TimerHandle(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_timer(
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
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_cancel_timer(ptr, handle.ptr),
            "cancel_timer",
        )
    }

    @JvmOverloads
    fun createWallTimer(
        periodSecs: Double,
        callback: TimerCallback,
        group: CallbackGroup? = null,
    ): TimerHandle = createTimer(periodSecs, callback, group)

    @JvmOverloads
    fun createService(
        serviceName: String,
        handler: ServiceHandler,
        group: CallbackGroup? = null,
    ): ServiceHandle {
        val cb =
            RobotBusC.ServiceCb { data, len, outData, outLen, _ ->
                try {
                    val body =
                        if (data != null && len > 0) {
                            data.getByteArray(0, len.toInt())
                        } else {
                            ByteArray(0)
                        }
                    val reply = handler.handle(body)
                    outLen.value = reply.size.toLong()
                    outData.value = NativeUtils.allocReplyBytes(reply)
                    0
                } catch (_: Exception) {
                    outData.value = null
                    outLen.value = 0
                    -1
                }
            }
        serviceHandlers.add(cb)
        return ServiceHandle(
            ptr,
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_service(
                    ptr,
                    serviceName,
                    cb,
                    null,
                    group?.raw(),
                ),
                "create_service",
            ),
        )
    }

    /** Typed service: handler receives / returns protobuf messages. */
    @JvmOverloads
    fun <Req : MessageLite, Resp : MessageLite> createService(
        serviceName: String,
        handler: TypedServiceHandler<Req, Resp>,
        requestType: Class<Req>,
        responseType: Class<Resp>,
        group: CallbackGroup? = null,
    ): ServiceHandle {
        ProtoCodec.requireMessageType(requestType, "requestType")
        ProtoCodec.requireMessageType(responseType, "responseType")
        @Suppress("UNCHECKED_CAST")
        val reqT = requestType as Class<Req>
        @Suppress("UNCHECKED_CAST")
        val respT = responseType as Class<Resp>
        return createService(
            serviceName,
            { body ->
                val req = ProtoCodec.tryParse(reqT, body) ?: return@createService ByteArray(0)
                val resp = handler.handle(req)
                if (!respT.isInstance(resp)) {
                    throw IllegalArgumentException(
                        "service handler must return ${respT.simpleName}, got ${resp.javaClass.simpleName}",
                    )
                }
                ProtoCodec.encode(resp)
            },
            group,
        )
    }

    fun createClient(serviceName: String): ServiceClient =
        ServiceClient(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_client(ptr, serviceName),
                "create_client",
            ),
        )

    /** Typed service client: `call(Request) -> Response`. */
    fun <Req : MessageLite, Resp : MessageLite> createClient(
        serviceName: String,
        requestType: Class<Req>,
        responseType: Class<Resp>,
    ): TypedServiceClient<Req, Resp> {
        ProtoCodec.requireMessageType(requestType, "requestType")
        ProtoCodec.requireMessageType(responseType, "responseType")
        @Suppress("UNCHECKED_CAST")
        val reqT = requestType as Class<Req>
        @Suppress("UNCHECKED_CAST")
        val respT = responseType as Class<Resp>
        return TypedServiceClient(createClient(serviceName), reqT, respT)
    }

    @JvmOverloads
    fun createActionServer(
        actionName: String,
        handler: ActionHandler,
        group: CallbackGroup? = null,
    ): ActionServerHandle {
        val cb =
            RobotBusC.ActionCb { data, len, outPhases, outCount, _ ->
                try {
                    val body =
                        if (data != null && len > 0) {
                            data.getByteArray(0, len.toInt())
                        } else {
                            ByteArray(0)
                        }
                    val phases = handler.handle(body)
                    outCount.value = phases.size.toLong()
                    if (phases.isEmpty()) {
                        outPhases.value = null
                        return@ActionCb 0
                    }
                    val arr =
                        RobotBusC.Holder.INSTANCE.robot_bus_alloc_action_phases(phases.size.toLong())
                    if (arr == null) {
                        outPhases.value = null
                        outCount.value = 0
                        return@ActionCb -1
                    }
                    val stride = RobotBusC.ActionPhaseStruct().size().toLong()
                    for (i in phases.indices) {
                        val phase = phases[i]
                        val item = RobotBusC.ActionPhaseStruct(arr.share(i * stride))
                        item.phase = RobotBusC.Holder.INSTANCE.robot_bus_dup_string(phase.phase)
                        item.bodyLen = phase.body.size.toLong()
                        item.body = NativeUtils.allocReplyBytes(phase.body)
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
        return ActionServerHandle(
            ptr,
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_action_server(
                    ptr,
                    actionName,
                    cb,
                    null,
                    group?.raw(),
                ),
                "create_action_server",
            ),
        )
    }

    /**
     * Typed action server: handler receives a goal message and returns [TypedActionPhase]
     * list (typically FEEDBACK / RESULT).
     */
    @JvmOverloads
    fun <Goal : MessageLite, Feedback : MessageLite, Result : MessageLite> createActionServer(
        actionName: String,
        handler: TypedActionHandler<Goal>,
        goalType: Class<Goal>,
        feedbackType: Class<Feedback>,
        resultType: Class<Result>,
        group: CallbackGroup? = null,
    ): ActionServerHandle {
        ProtoCodec.requireMessageType(goalType, "goalType")
        ProtoCodec.requireMessageType(feedbackType, "feedbackType")
        ProtoCodec.requireMessageType(resultType, "resultType")
        @Suppress("UNCHECKED_CAST")
        val goalT = goalType as Class<Goal>
        return createActionServer(
            actionName,
            { payload ->
                val goal = ProtoCodec.tryParse(goalT, payload)
                if (goal == null) {
                    return@createActionServer listOf(ActionPhase("RESULT", ByteArray(0)))
                }
                val replies = handler.handle(goal)
                val out = ArrayList<ActionPhase>(replies.size)
                for (phase in replies) {
                    val phaseU = phase.phase.uppercase()
                    val body = phase.body
                    if (phaseU == "FEEDBACK" && !feedbackType.isInstance(body)) {
                        throw IllegalArgumentException(
                            "FEEDBACK must be ${feedbackType.simpleName}, got ${body.javaClass.simpleName}",
                        )
                    }
                    if (phaseU == "RESULT" && !resultType.isInstance(body)) {
                        throw IllegalArgumentException(
                            "RESULT must be ${resultType.simpleName}, got ${body.javaClass.simpleName}",
                        )
                    }
                    out.add(ActionPhase(phase.phase, ProtoCodec.encode(body)))
                }
                out
            },
            group,
        )
    }

    fun createActionClient(actionName: String): ActionClient =
        ActionClient(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_action_client(ptr, actionName),
                "create_action_client",
            ),
        )

    /** Typed action client: encode goal / decode FEEDBACK and RESULT. */
    fun <Goal : MessageLite, Feedback : MessageLite, Result : MessageLite> createActionClient(
        actionName: String,
        goalType: Class<Goal>,
        feedbackType: Class<Feedback>,
        resultType: Class<Result>,
    ): TypedActionClient<Goal, Feedback, Result> {
        ProtoCodec.requireMessageType(goalType, "goalType")
        ProtoCodec.requireMessageType(feedbackType, "feedbackType")
        ProtoCodec.requireMessageType(resultType, "resultType")
        @Suppress("UNCHECKED_CAST")
        val goalT = goalType as Class<Goal>
        @Suppress("UNCHECKED_CAST")
        val fbT = feedbackType as Class<Feedback>
        @Suppress("UNCHECKED_CAST")
        val resT = resultType as Class<Result>
        return TypedActionClient(createActionClient(actionName), goalT, fbT, resT)
    }

    fun connectActionClient() {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_connect_action_client(ptr),
            "connect_action_client",
        )
    }

    fun shutdownHandle(): ShutdownHandle =
        ShutdownHandle(
            Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_node_shutdown_handle(ptr),
                "shutdown_handle",
            ),
        )

    fun shutdown() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_shutdown(ptr), "shutdown")
    }

    /**
     * Process pending work once.
     *
     * @return true if work was processed
     */
    @JvmOverloads
    fun spinOnce(timeoutSecs: Double = -1.0): Boolean {
        val rc = RobotBusC.Holder.INSTANCE.robot_bus_node_spin_once(ptr, timeoutSecs)
        if (rc < 0) {
            Errors.check(rc, "spin_once")
        }
        return rc == 1
    }

    /** Wait for one message on [topic]; returns null on timeout. */
    @JvmOverloads
    fun waitForMessage(topic: String, timeoutSecs: Double = -1.0): ByteArray? {
        val outData = PointerByReference()
        val outLen = LongByReference()
        val rc =
            RobotBusC.Holder.INSTANCE.robot_bus_node_wait_for_message(
                ptr,
                topic,
                timeoutSecs,
                outData,
                outLen,
            )
        if (rc < 0) {
            Errors.check(rc, "wait_for_message")
        }
        if (rc == 0) {
            return null
        }
        return NativeUtils.readBytes(outData.value, outLen.value)
    }

    fun spin() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_spin(ptr), "spin")
    }

    fun start() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_start(ptr), "start")
    }

    fun stop() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_stop(ptr), "stop")
    }

    fun waitForShutdown() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_wait(ptr), "wait")
    }

    fun declareParameter(name: String, value: Any) {
        var owned: Pointer? = null
        try {
            val nativeValue = toNativeParameter(value)
            owned = nativeValue.stringValue
            Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_declare_parameter(ptr, name, nativeValue),
                "declare_parameter",
            )
        } finally {
            if (owned != null) {
                RobotBusC.Holder.INSTANCE.robot_bus_free_string(owned)
            }
        }
    }

    fun setParameter(name: String, value: Any) {
        var owned: Pointer? = null
        try {
            val nativeValue = toNativeParameter(value)
            owned = nativeValue.stringValue
            Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_set_parameter(ptr, name, nativeValue),
                "set_parameter",
            )
        } finally {
            if (owned != null) {
                RobotBusC.Holder.INSTANCE.robot_bus_free_string(owned)
            }
        }
    }

    fun getParameterValue(name: String): Any {
        val out = RobotBusC.ParameterValueStruct()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_get_parameter(ptr, name, out),
            "get_parameter",
        )
        return fromNativeParameter(out, takeString = true)
    }

    fun getParameter(name: String): Parameter = Parameter(name, getParameterValue(name))

    fun hasParameter(name: String): Boolean {
        val rc = RobotBusC.Holder.INSTANCE.robot_bus_node_has_parameter(ptr, name)
        if (rc < 0) {
            Errors.check(rc, "has_parameter")
        }
        return rc == 1
    }

    fun undeclareParameter(name: String) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_undeclare_parameter(ptr, name),
            "undeclare_parameter",
        )
    }

    @JvmOverloads
    fun listParameters(
        prefixes: Array<String>? = null,
        depth: Long = 0,
    ): ListParametersResult {
        val outNames = PointerByReference()
        val namesCount = LongByReference()
        val outPrefixes = PointerByReference()
        val prefixesCount = LongByReference()
        val prefixPtr =
            if (prefixes != null && prefixes.isNotEmpty()) {
                com.sun.jna.StringArray(prefixes)
            } else {
                null
            }
        val prefixCount = prefixes?.size?.toLong() ?: 0L
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_list_parameters(
                ptr,
                prefixPtr,
                prefixCount,
                depth,
                outNames,
                namesCount,
                outPrefixes,
                prefixesCount,
            ),
            "list_parameters",
        )
        return ListParametersResult(
            takeStringList(outNames.value, namesCount.value),
            takeStringList(outPrefixes.value, prefixesCount.value),
        )
    }

    fun listAllParameters(): List<Parameter> {
        val out = PointerByReference()
        val countRef = LongByReference()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_list_all_parameters(ptr, out, countRef),
            "list_all_parameters",
        )
        val count = countRef.value
        val base = out.value
        val result = ArrayList<Parameter>(count.toInt())
        if (base != null && count > 0) {
            val first = RobotBusC.ParameterStruct(base)
            val arr = first.toArray(count.toInt())
            for (s in arr) {
                val p = s as RobotBusC.ParameterStruct
                val pname = p.name?.getString(0) ?: ""
                val value = fromNativeParameter(p.value, takeString = false)
                result.add(Parameter(pname, value))
            }
            RobotBusC.Holder.INSTANCE.robot_bus_parameters_free(base, count)
        }
        return result
    }

    private fun takeStringList(base: Pointer?, count: Long): List<String> {
        val result = ArrayList<String>(count.toInt())
        if (base != null && count > 0) {
            for (i in 0 until count.toInt()) {
                val sp = base.getPointer((i * com.sun.jna.Native.POINTER_SIZE).toLong())
                result.add(sp?.getString(0) ?: "")
            }
            RobotBusC.Holder.INSTANCE.robot_bus_string_list_free(base, count)
        }
        return result
    }

    fun loadParametersFromYaml(path: String) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_load_parameters_from_yaml(ptr, path),
            "load_parameters_from_yaml",
        )
    }

    fun loadParametersFromYamlStr(yaml: String) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_node_load_parameters_from_yaml_str(ptr, yaml),
            "load_parameters_from_yaml_str",
        )
    }

    internal fun raw(): Pointer? = ptr

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_node_free(ptr)
        ptr = null
        msgCallbacks.clear()
        timerCallbacks.clear()
        serviceHandlers.clear()
        actionHandlers.clear()
    }

    companion object {
        @JvmStatic
        @JvmOverloads
        fun tcp(name: String, host: String = "localhost"): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_tcp(name, host),
                    "Node.tcp",
                ),
            )

        @JvmStatic
        @JvmOverloads
        fun ipc(name: String, path: String? = null): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_ipc(name, path),
                    "Node.ipc",
                ),
            )

        @JvmStatic
        @JvmOverloads
        fun inproc(name: String, prefix: String? = null): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_inproc(name, prefix),
                    "Node.inproc",
                ),
            )

        /** Same-process inproc sharing [context] with an embedded broker. */
        @JvmStatic
        @JvmOverloads
        fun inproc(context: Context, name: String, prefix: String? = null): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_inproc_with_context(
                        context.raw(),
                        name,
                        prefix,
                    ),
                    "Node.inproc",
                ),
            )

        @JvmStatic
        @JvmOverloads
        fun withContext(context: Context, name: String, options: NodeOptions? = null): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_new_with_context(
                        context.raw(),
                        name,
                        options?.toNative(),
                    ),
                    "Node.withContext",
                ),
            )

        @JvmStatic
        fun ws(name: String): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_ws(name),
                    "Node.ws",
                ),
            )

        @JvmStatic
        fun wsAt(name: String, url: String): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_ws_at(name, url),
                    "Node.wsAt",
                ),
            )

        /** Discover a broker via HTTP `GET /api/v1/discover`, then connect with [transport]. */
        @JvmStatic
        @JvmOverloads
        fun discover(name: String, transport: String = "tcp", opts: DiscoverOpts? = null): Node =
            Node(
                Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_discover(
                        name,
                        transport,
                        opts?.toNative(),
                    ),
                    "Node.discover",
                ),
            )

        @JvmStatic
        internal fun fromRaw(ptr: Pointer): Node = Node(ptr)

        private fun toNativeParameter(value: Any): RobotBusC.ParameterValueStruct {
            val v = RobotBusC.ParameterValueStruct()
            when (value) {
                is Boolean -> {
                    v.type = RobotBusC.ParameterValueStruct.TYPE_BOOL
                    v.boolValue = if (value) 1 else 0
                }
                is Int -> {
                    v.type = RobotBusC.ParameterValueStruct.TYPE_INTEGER
                    v.integerValue = value.toLong()
                }
                is Long -> {
                    v.type = RobotBusC.ParameterValueStruct.TYPE_INTEGER
                    v.integerValue = value
                }
                is Float -> {
                    v.type = RobotBusC.ParameterValueStruct.TYPE_DOUBLE
                    v.doubleValue = value.toDouble()
                }
                is Double -> {
                    v.type = RobotBusC.ParameterValueStruct.TYPE_DOUBLE
                    v.doubleValue = value
                }
                is String -> {
                    v.type = RobotBusC.ParameterValueStruct.TYPE_STRING
                    v.stringValue = RobotBusC.Holder.INSTANCE.robot_bus_dup_string(value)
                }
                else ->
                    throw IllegalArgumentException(
                        "parameter value must be boolean, int/long, float/double, or String",
                    )
            }
            return v
        }

        private fun fromNativeParameter(
            v: RobotBusC.ParameterValueStruct,
            takeString: Boolean,
        ): Any =
            when (v.type) {
                RobotBusC.ParameterValueStruct.TYPE_BOOL -> v.boolValue != 0
                RobotBusC.ParameterValueStruct.TYPE_INTEGER -> v.integerValue
                RobotBusC.ParameterValueStruct.TYPE_DOUBLE -> v.doubleValue
                RobotBusC.ParameterValueStruct.TYPE_STRING ->
                    if (takeString) {
                        NativeUtils.takeCString(v.stringValue)
                    } else {
                        v.stringValue?.getString(0) ?: ""
                    }
                else -> throw RobotBusException("unknown parameter type: ${v.type}")
            }
    }
}
