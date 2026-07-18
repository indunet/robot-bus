package org.indunet.robot.bus;

import com.sun.jna.Pointer;
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

    public ServiceClient createClient(String serviceName) {
        return new ServiceClient(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_create_client(ptr, serviceName),
                        "create_client"));
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

    public ActionClient createActionClient(String actionName) {
        return new ActionClient(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_node_create_action_client(ptr, actionName),
                        "create_action_client"));
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
