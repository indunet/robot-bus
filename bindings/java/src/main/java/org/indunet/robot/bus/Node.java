package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

/** Robot-bus node: topics, services, actions, timers, and spinning. */
public final class Node implements AutoCloseable {
    private Pointer ptr;
    private final CopyOnWriteArrayList<RobotBusC.MsgCb> msgCallbacks = new CopyOnWriteArrayList<>();
    private final CopyOnWriteArrayList<RobotBusC.TimerCb> timerCallbacks = new CopyOnWriteArrayList<>();
    private final CopyOnWriteArrayList<RobotBusC.ServiceCb> serviceHandlers = new CopyOnWriteArrayList<>();
    private final CopyOnWriteArrayList<RobotBusC.ActionCb> actionHandlers = new CopyOnWriteArrayList<>();

    public Node(String name) {
        this(name, null);
    }

    public Node(String name, NodeOptions options) {
        this(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_new(
                                name, options != null ? options.toNative() : null),
                        "Node"));
    }

    private Node(Pointer ptr) {
        this.ptr = ptr;
    }

    public static Node tcp(String name) {
        return tcp(name, "localhost");
    }

    public static Node tcp(String name, String host) {
        return new Node(Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_node_tcp(name, host), "Node.tcp"));
    }

    public static Node ipc(String name) {
        return ipc(name, null);
    }

    public static Node ipc(String name, String path) {
        return new Node(Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_node_ipc(name, path), "Node.ipc"));
    }

    public static Node inproc(String name) {
        return inproc(name, null);
    }

    public static Node inproc(String name, String prefix) {
        return new Node(
                Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_node_inproc(name, prefix), "Node.inproc"));
    }

    /** Same-process inproc sharing {@code context} with an embedded broker. */
    public static Node inproc(Context context, String name) {
        return inproc(context, name, null);
    }

    public static Node inproc(Context context, String name, String prefix) {
        return new Node(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_inproc_with_context(
                                context.raw(), name, prefix),
                        "Node.inproc"));
    }

    public static Node withContext(Context context, String name) {
        return withContext(context, name, null);
    }

    public static Node withContext(Context context, String name, NodeOptions options) {
        return new Node(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_new_with_context(
                                context.raw(), name, options != null ? options.toNative() : null),
                        "Node.withContext"));
    }

    public static Node grpc(String name) {
        return new Node(Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_node_grpc(name), "Node.grpc"));
    }

    public static Node grpcAt(String name, String url) {
        return new Node(
                Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_node_grpc_at(name, url), "Node.grpcAt"));
    }

    static Node fromRaw(Pointer ptr) {
        return new Node(ptr);
    }

    public String name() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_node_name(ptr));
    }

    public CallbackGroup createCallbackGroup() {
        return createCallbackGroup(CallbackGroupType.MutuallyExclusive);
    }

    public CallbackGroup createCallbackGroup(CallbackGroupType kind) {
        return new CallbackGroup(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_create_callback_group(ptr, kind.getCode()),
                        "create_callback_group"));
    }

    public TopicPublisher createPublisher(String topic) {
        return new TopicPublisher(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_create_publisher(ptr, topic),
                        "create_publisher"));
    }

    /** Typed publisher: {@code publish(Message)} with automatic protobuf encode. */
    public <T extends MessageLite> TypedTopicPublisher<T> createPublisher(String topic, Class<T> msgType) {
        ProtoCodec.requireMessageType(msgType, "msgType");
        @SuppressWarnings("unchecked")
        Class<T> typed = (Class<T>) msgType;
        return new TypedTopicPublisher<>(createPublisher(topic), typed);
    }

    public void createSubscription(String topic, MsgCallback callback) {
        createSubscription(topic, callback, null);
    }

    public void createSubscription(String topic, MsgCallback callback, CallbackGroup group) {
        RobotBusC.MsgCb cb =
                (t, data, len, user) -> {
                    byte[] bytes =
                            (data != null && len > 0) ? data.getByteArray(0, (int) len) : new byte[0];
                    callback.onMessage(t != null ? t : "", bytes);
                };
        msgCallbacks.add(cb);
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_subscription(
                        ptr, topic, cb, null, group != null ? group.raw() : null),
                "create_subscription");
    }

    /** Typed subscription: callback receives a decoded protobuf message. */
    public <T extends MessageLite> void createSubscription(
            String topic, TypedMsgCallback<T> callback, Class<T> msgType) {
        createSubscription(topic, callback, msgType, null);
    }

    public <T extends MessageLite> void createSubscription(
            String topic, TypedMsgCallback<T> callback, Class<T> msgType, CallbackGroup group) {
        ProtoCodec.requireMessageType(msgType, "msgType");
        @SuppressWarnings("unchecked")
        Class<T> typed = (Class<T>) msgType;
        createSubscription(
                topic,
                (t, payload) -> {
                    T msg = ProtoCodec.tryParse(typed, payload);
                    if (msg == null) {
                        return;
                    }
                    callback.onMessage(t, msg);
                },
                group);
    }

    public TimerHandle createTimer(double periodSecs, TimerCallback callback) {
        return createTimer(periodSecs, callback, null);
    }

    public TimerHandle createTimer(double periodSecs, TimerCallback callback, CallbackGroup group) {
        RobotBusC.TimerCb cb = user -> callback.onTimer();
        timerCallbacks.add(cb);
        return new TimerHandle(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_create_timer(
                                ptr, periodSecs, cb, null, group != null ? group.raw() : null),
                        "create_timer"));
    }

    public void cancelTimer(TimerHandle handle) {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_cancel_timer(ptr, handle.ptr), "cancel_timer");
    }

    public void createService(String serviceName, ServiceHandler handler) {
        createService(serviceName, handler, null);
    }

    public void createService(String serviceName, ServiceHandler handler, CallbackGroup group) {
        RobotBusC.ServiceCb cb =
                (data, len, outData, outLen, user) -> {
                    try {
                        byte[] body =
                                (data != null && len > 0)
                                        ? data.getByteArray(0, (int) len)
                                        : new byte[0];
                        byte[] reply = handler.handle(body);
                        outLen.setValue(reply.length);
                        outData.setValue(NativeUtils.allocReplyBytes(reply));
                        return 0;
                    } catch (Exception e) {
                        outData.setValue(null);
                        outLen.setValue(0);
                        return -1;
                    }
                };
        serviceHandlers.add(cb);
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_service(
                        ptr, serviceName, cb, null, group != null ? group.raw() : null),
                "create_service");
    }

    /** Typed service: handler receives / returns protobuf messages. */
    public <Req extends MessageLite, Resp extends MessageLite> void createService(
            String serviceName,
            TypedServiceHandler<Req, Resp> handler,
            Class<Req> requestType,
            Class<Resp> responseType) {
        createService(serviceName, handler, requestType, responseType, null);
    }

    public <Req extends MessageLite, Resp extends MessageLite> void createService(
            String serviceName,
            TypedServiceHandler<Req, Resp> handler,
            Class<Req> requestType,
            Class<Resp> responseType,
            CallbackGroup group) {
        ProtoCodec.requireMessageType(requestType, "requestType");
        ProtoCodec.requireMessageType(responseType, "responseType");
        @SuppressWarnings("unchecked")
        Class<Req> reqT = (Class<Req>) requestType;
        @SuppressWarnings("unchecked")
        Class<Resp> respT = (Class<Resp>) responseType;
        createService(
                serviceName,
                body -> {
                    Req req = ProtoCodec.tryParse(reqT, body);
                    if (req == null) {
                        return new byte[0];
                    }
                    Resp resp = handler.handle(req);
                    if (resp == null || !respT.isInstance(resp)) {
                        throw new IllegalArgumentException(
                                "service handler must return "
                                        + respT.getSimpleName()
                                        + ", got "
                                        + (resp == null ? "null" : resp.getClass().getSimpleName()));
                    }
                    return ProtoCodec.encode(resp);
                },
                group);
    }

    public ServiceClient createClient(String serviceName) {
        return new ServiceClient(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_create_client(ptr, serviceName),
                        "create_client"));
    }

    /** Typed service client: {@code call(Request) -> Response}. */
    public <Req extends MessageLite, Resp extends MessageLite> TypedServiceClient<Req, Resp> createClient(
            String serviceName, Class<Req> requestType, Class<Resp> responseType) {
        ProtoCodec.requireMessageType(requestType, "requestType");
        ProtoCodec.requireMessageType(responseType, "responseType");
        @SuppressWarnings("unchecked")
        Class<Req> reqT = (Class<Req>) requestType;
        @SuppressWarnings("unchecked")
        Class<Resp> respT = (Class<Resp>) responseType;
        return new TypedServiceClient<>(createClient(serviceName), reqT, respT);
    }

    public void createActionServer(String actionName, ActionHandler handler) {
        createActionServer(actionName, handler, null);
    }

    public void createActionServer(String actionName, ActionHandler handler, CallbackGroup group) {
        RobotBusC.ActionCb cb =
                (data, len, outPhases, outCount, user) -> {
                    try {
                        byte[] body =
                                (data != null && len > 0)
                                        ? data.getByteArray(0, (int) len)
                                        : new byte[0];
                        List<ActionPhase> phases = handler.handle(body);
                        outCount.setValue(phases.size());
                        if (phases.isEmpty()) {
                            outPhases.setValue(null);
                            return 0;
                        }
                        Pointer arr =
                                RobotBusC.Holder.INSTANCE.robot_bus_alloc_action_phases(phases.size());
                        if (arr == null) {
                            outPhases.setValue(null);
                            outCount.setValue(0);
                            return -1;
                        }
                        long stride = new RobotBusC.ActionPhaseStruct().size();
                        for (int i = 0; i < phases.size(); i++) {
                            ActionPhase phase = phases.get(i);
                            RobotBusC.ActionPhaseStruct item =
                                    new RobotBusC.ActionPhaseStruct(arr.share(i * stride));
                            item.phase = RobotBusC.Holder.INSTANCE.robot_bus_dup_string(phase.getPhase());
                            item.bodyLen = phase.getBody().length;
                            item.body = NativeUtils.allocReplyBytes(phase.getBody());
                            item.write();
                        }
                        outPhases.setValue(arr);
                        return 0;
                    } catch (Exception e) {
                        outPhases.setValue(null);
                        outCount.setValue(0);
                        return -1;
                    }
                };
        actionHandlers.add(cb);
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_create_action_server(
                        ptr, actionName, cb, null, group != null ? group.raw() : null),
                "create_action_server");
    }

    /**
     * Typed action server: handler receives a goal message and returns {@link TypedActionPhase}
     * list (typically FEEDBACK / RESULT).
     */
    public <Goal extends MessageLite, Feedback extends MessageLite, Result extends MessageLite>
            void createActionServer(
                    String actionName,
                    TypedActionHandler<Goal> handler,
                    Class<Goal> goalType,
                    Class<Feedback> feedbackType,
                    Class<Result> resultType) {
        createActionServer(actionName, handler, goalType, feedbackType, resultType, null);
    }

    public <Goal extends MessageLite, Feedback extends MessageLite, Result extends MessageLite>
            void createActionServer(
                    String actionName,
                    TypedActionHandler<Goal> handler,
                    Class<Goal> goalType,
                    Class<Feedback> feedbackType,
                    Class<Result> resultType,
                    CallbackGroup group) {
        ProtoCodec.requireMessageType(goalType, "goalType");
        ProtoCodec.requireMessageType(feedbackType, "feedbackType");
        ProtoCodec.requireMessageType(resultType, "resultType");
        @SuppressWarnings("unchecked")
        Class<Goal> goalT = (Class<Goal>) goalType;
        createActionServer(
                actionName,
                payload -> {
                    Goal goal = ProtoCodec.tryParse(goalT, payload);
                    if (goal == null) {
                        return List.of(new ActionPhase("RESULT", new byte[0]));
                    }
                    List<TypedActionPhase> replies = handler.handle(goal);
                    List<ActionPhase> out = new ArrayList<>(replies.size());
                    for (TypedActionPhase phase : replies) {
                        String phaseU = phase.getPhase() != null ? phase.getPhase().toUpperCase() : "";
                        MessageLite body = phase.getBody();
                        if ("FEEDBACK".equals(phaseU) && !feedbackType.isInstance(body)) {
                            throw new IllegalArgumentException(
                                    "FEEDBACK must be "
                                            + feedbackType.getSimpleName()
                                            + ", got "
                                            + body.getClass().getSimpleName());
                        }
                        if ("RESULT".equals(phaseU) && !resultType.isInstance(body)) {
                            throw new IllegalArgumentException(
                                    "RESULT must be "
                                            + resultType.getSimpleName()
                                            + ", got "
                                            + body.getClass().getSimpleName());
                        }
                        out.add(new ActionPhase(phase.getPhase(), ProtoCodec.encode(body)));
                    }
                    return out;
                },
                group);
    }

    public ActionClient createActionClient(String actionName) {
        return new ActionClient(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_create_action_client(ptr, actionName),
                        "create_action_client"));
    }

    /** Typed action client: encode goal / decode FEEDBACK and RESULT. */
    public <Goal extends MessageLite, Feedback extends MessageLite, Result extends MessageLite>
            TypedActionClient<Goal, Feedback, Result> createActionClient(
                    String actionName,
                    Class<Goal> goalType,
                    Class<Feedback> feedbackType,
                    Class<Result> resultType) {
        ProtoCodec.requireMessageType(goalType, "goalType");
        ProtoCodec.requireMessageType(feedbackType, "feedbackType");
        ProtoCodec.requireMessageType(resultType, "resultType");
        @SuppressWarnings("unchecked")
        Class<Goal> goalT = (Class<Goal>) goalType;
        @SuppressWarnings("unchecked")
        Class<Feedback> fbT = (Class<Feedback>) feedbackType;
        @SuppressWarnings("unchecked")
        Class<Result> resT = (Class<Result>) resultType;
        return new TypedActionClient<>(createActionClient(actionName), goalT, fbT, resT);
    }

    public void connectActionClient() {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_connect_action_client(ptr),
                "connect_action_client");
    }

    public ShutdownHandle shutdownHandle() {
        return new ShutdownHandle(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_shutdown_handle(ptr), "shutdown_handle"));
    }

    public void shutdown() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_shutdown(ptr), "shutdown");
    }

    /**
     * Process pending work once.
     *
     * @return true if work was processed
     */
    public boolean spinOnce() {
        return spinOnce(-1.0);
    }

    public boolean spinOnce(double timeoutSecs) {
        int rc = RobotBusC.Holder.INSTANCE.robot_bus_node_spin_once(ptr, timeoutSecs);
        if (rc < 0) {
            Errors.check(rc, "spin_once");
        }
        return rc == 1;
    }

    public void spin() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_spin(ptr), "spin");
    }

    public void start() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_start(ptr), "start");
    }

    public void stop() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_stop(ptr), "stop");
    }

    public void waitForShutdown() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_node_wait(ptr), "wait");
    }

    public void declareParameter(String name, Object value) {
        Pointer owned = null;
        try {
            RobotBusC.ParameterValueStruct nativeValue = toNativeParameter(value);
            owned = nativeValue.stringValue;
            Errors.check(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_declare_parameter(ptr, name, nativeValue),
                    "declare_parameter");
        } finally {
            if (owned != null) {
                RobotBusC.Holder.INSTANCE.robot_bus_free_string(owned);
            }
        }
    }

    public void setParameter(String name, Object value) {
        Pointer owned = null;
        try {
            RobotBusC.ParameterValueStruct nativeValue = toNativeParameter(value);
            owned = nativeValue.stringValue;
            Errors.check(
                    RobotBusC.Holder.INSTANCE.robot_bus_node_set_parameter(ptr, name, nativeValue),
                    "set_parameter");
        } finally {
            if (owned != null) {
                RobotBusC.Holder.INSTANCE.robot_bus_free_string(owned);
            }
        }
    }

    public Object getParameter(String name) {
        RobotBusC.ParameterValueStruct out = new RobotBusC.ParameterValueStruct();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_get_parameter(ptr, name, out),
                "get_parameter");
        return fromNativeParameter(out, true);
    }

    public boolean hasParameter(String name) {
        int rc = RobotBusC.Holder.INSTANCE.robot_bus_node_has_parameter(ptr, name);
        if (rc < 0) {
            Errors.check(rc, "has_parameter");
        }
        return rc == 1;
    }

    public List<Parameter> listParameters() {
        PointerByReference out = new PointerByReference();
        LongByReference countRef = new LongByReference();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_list_parameters(ptr, out, countRef),
                "list_parameters");
        long count = countRef.getValue();
        Pointer base = out.getValue();
        List<Parameter> result = new ArrayList<>((int) count);
        if (base != null && count > 0) {
            RobotBusC.ParameterStruct first = new RobotBusC.ParameterStruct(base);
            Structure[] arr = first.toArray((int) count);
            for (Structure s : arr) {
                RobotBusC.ParameterStruct p = (RobotBusC.ParameterStruct) s;
                String pname = p.name != null ? p.name.getString(0) : "";
                Object value = fromNativeParameter(p.value, false);
                result.add(new Parameter(pname, value));
            }
            RobotBusC.Holder.INSTANCE.robot_bus_parameters_free(base, count);
        }
        return result;
    }

    public void loadParametersFromYaml(String path) {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_load_parameters_from_yaml(ptr, path),
                "load_parameters_from_yaml");
    }

    public void loadParametersFromYamlStr(String yaml) {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_node_load_parameters_from_yaml_str(ptr, yaml),
                "load_parameters_from_yaml_str");
    }

    private static RobotBusC.ParameterValueStruct toNativeParameter(Object value) {
        RobotBusC.ParameterValueStruct v = new RobotBusC.ParameterValueStruct();
        if (value instanceof Boolean) {
            v.type = RobotBusC.ParameterValueStruct.TYPE_BOOL;
            v.boolValue = Boolean.TRUE.equals(value) ? 1 : 0;
        } else if (value instanceof Integer) {
            v.type = RobotBusC.ParameterValueStruct.TYPE_INTEGER;
            v.integerValue = ((Integer) value).longValue();
        } else if (value instanceof Long) {
            v.type = RobotBusC.ParameterValueStruct.TYPE_INTEGER;
            v.integerValue = (Long) value;
        } else if (value instanceof Float) {
            v.type = RobotBusC.ParameterValueStruct.TYPE_DOUBLE;
            v.doubleValue = ((Float) value).doubleValue();
        } else if (value instanceof Double) {
            v.type = RobotBusC.ParameterValueStruct.TYPE_DOUBLE;
            v.doubleValue = (Double) value;
        } else if (value instanceof String) {
            v.type = RobotBusC.ParameterValueStruct.TYPE_STRING;
            v.stringValue = RobotBusC.Holder.INSTANCE.robot_bus_dup_string((String) value);
        } else {
            throw new IllegalArgumentException(
                    "parameter value must be boolean, int/long, float/double, or String");
        }
        return v;
    }

    private static Object fromNativeParameter(RobotBusC.ParameterValueStruct v, boolean takeString) {
        switch (v.type) {
            case RobotBusC.ParameterValueStruct.TYPE_BOOL:
                return v.boolValue != 0;
            case RobotBusC.ParameterValueStruct.TYPE_INTEGER:
                return v.integerValue;
            case RobotBusC.ParameterValueStruct.TYPE_DOUBLE:
                return v.doubleValue;
            case RobotBusC.ParameterValueStruct.TYPE_STRING:
                if (takeString) {
                    return NativeUtils.takeCString(v.stringValue);
                }
                return v.stringValue != null ? v.stringValue.getString(0) : "";
            default:
                throw new RobotBusException("unknown parameter type: " + v.type);
        }
    }

    Pointer raw() {
        return ptr;
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_node_free(ptr);
        ptr = null;
        msgCallbacks.clear();
        timerCallbacks.clear();
        serviceHandlers.clear();
        actionHandlers.clear();
    }
}
