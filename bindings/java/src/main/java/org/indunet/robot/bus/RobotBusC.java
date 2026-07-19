package org.indunet.robot.bus;

import com.sun.jna.Callback;
import com.sun.jna.Library;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;

/** JNA mapping of {@code bindings/cpp/include/robot_bus.h}. */
interface RobotBusC extends Library {

    final class Holder {
        static final RobotBusC INSTANCE = NativeLoader.loadLibrary();

        private Holder() {}
    }

    String robot_bus_last_error();

    void robot_bus_free_string(Pointer s);

    void robot_bus_free_bytes(Pointer data, long len);

    Pointer robot_bus_alloc_bytes(long len);

    Pointer robot_bus_dup_string(String s);

    Pointer robot_bus_alloc_action_phases(long count);

    void robot_bus_action_messages_free(Pointer msgs, long count);

    void robot_bus_action_phases_free(Pointer phases, long count);

    int robot_bus_message_xsub_endpoint(String host, String transport, PointerByReference out);

    int robot_bus_message_xpub_endpoint(String host, String transport, PointerByReference out);

    Pointer robot_bus_publisher_new(String endpoint);

    void robot_bus_publisher_free(Pointer p);

    int robot_bus_publisher_publish(Pointer p, String topic, byte[] data, long len);

    Pointer robot_bus_publisher_endpoint(Pointer p);

    Pointer robot_bus_subscriber_new(String endpoint);

    void robot_bus_subscriber_free(Pointer s);

    int robot_bus_subscriber_subscribe(Pointer s, String topic);

    int robot_bus_subscriber_unsubscribe(Pointer s, String topic);

    int robot_bus_subscriber_receive(
            Pointer s,
            double timeoutSecs,
            PointerByReference outTopic,
            PointerByReference outData,
            LongByReference outLen);

    Pointer robot_bus_subscriber_endpoint(Pointer s);

    void robot_bus_shutdown_handle_free(Pointer h);

    void robot_bus_shutdown_handle_shutdown(Pointer h);

    int robot_bus_shutdown_handle_is_running(Pointer h);

    void robot_bus_timer_handle_free(Pointer h);

    void robot_bus_callback_group_free(Pointer g);

    long robot_bus_callback_group_id(Pointer g);

    int robot_bus_callback_group_kind(Pointer g);

    void robot_bus_topic_publisher_free(Pointer p);

    Pointer robot_bus_topic_publisher_topic(Pointer p);

    int robot_bus_topic_publisher_publish(Pointer p, byte[] data, long len);

    void robot_bus_service_client_free(Pointer c);

    Pointer robot_bus_service_client_service_name(Pointer c);

    int robot_bus_service_client_call(
            Pointer c,
            byte[] data,
            long len,
            double timeoutSecs,
            PointerByReference outData,
            LongByReference outLen);

    void robot_bus_action_client_free(Pointer c);

    Pointer robot_bus_action_client_action_name(Pointer c);

    int robot_bus_action_client_send_goal(
            Pointer c,
            byte[] data,
            long len,
            String goalId,
            double timeoutSecs,
            PointerByReference outMsgs,
            LongByReference outCount);

    int robot_bus_action_client_cancel(
            Pointer c,
            String goalId,
            byte[] data,
            long len,
            double timeoutSecs,
            ActionMessageStruct outMsg);

    Pointer robot_bus_context_new();

    void robot_bus_context_free(Pointer c);

    Pointer robot_bus_context_clone(Pointer c);

    Pointer robot_bus_node_new(String name, NodeOptions opts);

    Pointer robot_bus_node_new_with_context(Pointer ctx, String name, NodeOptions opts);

    Pointer robot_bus_node_tcp(String name, String host);

    Pointer robot_bus_node_ipc(String name, String path);

    Pointer robot_bus_node_inproc(String name, String prefix);

    Pointer robot_bus_node_inproc_with_context(Pointer ctx, String name, String prefix);

    Pointer robot_bus_node_grpc(String name);

    Pointer robot_bus_node_grpc_at(String name, String url);

    void robot_bus_node_free(Pointer n);

    Pointer robot_bus_node_name(Pointer n);

    Pointer robot_bus_node_create_callback_group(Pointer n, int kind);

    Pointer robot_bus_node_create_publisher(Pointer n, String topic);

    int robot_bus_node_create_subscription(
            Pointer n, String topic, MsgCb callback, Pointer user, Pointer group);

    Pointer robot_bus_node_create_timer(
            Pointer n, double periodSecs, TimerCb callback, Pointer user, Pointer group);

    int robot_bus_node_cancel_timer(Pointer n, Pointer handle);

    int robot_bus_node_create_service(
            Pointer n, String serviceName, ServiceCb handler, Pointer user, Pointer group);

    Pointer robot_bus_node_create_client(Pointer n, String serviceName);

    int robot_bus_node_create_action_server(
            Pointer n, String actionName, ActionCb handler, Pointer user, Pointer group);

    Pointer robot_bus_node_create_action_client(Pointer n, String actionName);

    int robot_bus_node_connect_action_client(Pointer n);

    Pointer robot_bus_node_shutdown_handle(Pointer n);

    int robot_bus_node_shutdown(Pointer n);

    int robot_bus_node_spin_once(Pointer n, double timeoutSecs);

    int robot_bus_node_spin(Pointer n);

    int robot_bus_node_start(Pointer n);

    int robot_bus_node_stop(Pointer n);

    int robot_bus_node_wait(Pointer n);

    int robot_bus_node_declare_parameter(Pointer n, String name, ParameterValueStruct value);

    int robot_bus_node_set_parameter(Pointer n, String name, ParameterValueStruct value);

    int robot_bus_node_get_parameter(Pointer n, String name, ParameterValueStruct out);

    int robot_bus_node_has_parameter(Pointer n, String name);

    int robot_bus_node_list_parameters(Pointer n, PointerByReference out, LongByReference outCount);

    void robot_bus_parameters_free(Pointer params, long count);

    int robot_bus_node_load_parameters_from_yaml(Pointer n, String path);

    int robot_bus_node_load_parameters_from_yaml_str(Pointer n, String yaml);

    Pointer robot_bus_single_threaded_executor_new();

    Pointer robot_bus_single_threaded_executor_new_with_context(Pointer ctx);

    void robot_bus_single_threaded_executor_free(Pointer e);

    int robot_bus_single_threaded_executor_add_node(Pointer e, Pointer n);

    Pointer robot_bus_single_threaded_executor_create_node(Pointer e, String name, NodeOptions opts);

    Pointer robot_bus_single_threaded_executor_shutdown_handle(Pointer e);

    int robot_bus_single_threaded_executor_shutdown(Pointer e);

    int robot_bus_single_threaded_executor_spin_once(Pointer e, double timeoutSecs);

    int robot_bus_single_threaded_executor_spin(Pointer e);

    int robot_bus_single_threaded_executor_start(Pointer e);

    int robot_bus_single_threaded_executor_stop(Pointer e);

    int robot_bus_single_threaded_executor_wait(Pointer e);

    Pointer robot_bus_multi_threaded_executor_new(long numThreads);

    Pointer robot_bus_multi_threaded_executor_new_with_context(Pointer ctx, long numThreads);

    void robot_bus_multi_threaded_executor_free(Pointer e);

    int robot_bus_multi_threaded_executor_add_node(Pointer e, Pointer n);

    Pointer robot_bus_multi_threaded_executor_create_node(Pointer e, String name, NodeOptions opts);

    Pointer robot_bus_multi_threaded_executor_shutdown_handle(Pointer e);

    int robot_bus_multi_threaded_executor_shutdown(Pointer e);

    int robot_bus_multi_threaded_executor_spin_once(Pointer e, double timeoutSecs);

    int robot_bus_multi_threaded_executor_spin(Pointer e);

    Pointer robot_bus_broker_start(BrokerOptions opts);

    Pointer robot_bus_broker_start_with_context(Pointer ctx, BrokerOptions opts);

    void robot_bus_broker_free(Pointer b);

    int robot_bus_broker_stop(Pointer b);

    Pointer robot_bus_broker_message_xsub_bind(Pointer b);

    Pointer robot_bus_broker_message_xpub_bind(Pointer b);

    Pointer robot_bus_broker_service_frontend_bind(Pointer b);

    Pointer robot_bus_broker_service_backend_bind(Pointer b);

    Pointer robot_bus_broker_action_frontend_bind(Pointer b);

    Pointer robot_bus_broker_action_backend_bind(Pointer b);

    Pointer robot_bus_broker_grpc_listen(Pointer b);

    Pointer robot_bus_broker_console_listen(Pointer b);

    interface MsgCb extends Callback {
        void invoke(String topic, Pointer data, long len, Pointer user);
    }

    interface TimerCb extends Callback {
        void invoke(Pointer user);
    }

    interface ServiceCb extends Callback {
        int invoke(
                Pointer data,
                long len,
                PointerByReference outData,
                LongByReference outLen,
                Pointer user);
    }

    interface ActionCb extends Callback {
        int invoke(
                Pointer data,
                long len,
                PointerByReference outPhases,
                LongByReference outCount,
                Pointer user);
    }

    @Structure.FieldOrder({"type", "boolValue", "integerValue", "doubleValue", "stringValue"})
    class ParameterValueStruct extends Structure {
        public static final int TYPE_BOOL = 0;
        public static final int TYPE_INTEGER = 1;
        public static final int TYPE_DOUBLE = 2;
        public static final int TYPE_STRING = 3;

        public int type;
        public int boolValue;
        public long integerValue;
        public double doubleValue;
        public Pointer stringValue;

        public ParameterValueStruct() {
            super();
        }

        public ParameterValueStruct(Pointer p) {
            super(p);
            read();
        }
    }

    @Structure.FieldOrder({"name", "value"})
    class ParameterStruct extends Structure {
        public Pointer name;
        public ParameterValueStruct value;

        public ParameterStruct() {
            super();
            value = new ParameterValueStruct();
        }

        public ParameterStruct(Pointer p) {
            super(p);
            value = new ParameterValueStruct();
            read();
        }
    }

    @Structure.FieldOrder({
        "host",
        "transport",
        "grpcUrl",
        "messageXsub",
        "messageXpub",
        "serviceFrontend",
        "serviceBackend",
        "actionBackend",
        "actionFrontend"
    })
    class NodeOptions extends Structure {
        public String host;
        public String transport;
        public String grpcUrl;
        public String messageXsub;
        public String messageXpub;
        public String serviceFrontend;
        public String serviceBackend;
        public String actionBackend;
        public String actionFrontend;
    }

    @Structure.FieldOrder({
        "messageXsubBind",
        "messageXpubBind",
        "serviceFrontendBind",
        "serviceBackendBind",
        "actionFrontendBind",
        "actionBackendBind",
        "grpcListen",
        "consoleListen",
        "tcpOnly",
        "noConsole"
    })
    class BrokerOptions extends Structure {
        public String messageXsubBind;
        public String messageXpubBind;
        public String serviceFrontendBind;
        public String serviceBackendBind;
        public String actionFrontendBind;
        public String actionBackendBind;
        public String grpcListen;
        public String consoleListen;
        public int tcpOnly;
        public int noConsole;
    }

    @Structure.FieldOrder({"kind", "body", "bodyLen", "goalId", "actionName"})
    class ActionMessageStruct extends Structure {
        public Pointer kind;
        public Pointer body;
        public long bodyLen;
        public Pointer goalId;
        public Pointer actionName;

        public ActionMessageStruct() {
            super();
        }

        public ActionMessageStruct(Pointer p) {
            super(p);
            read();
        }
    }

    @Structure.FieldOrder({"phase", "body", "bodyLen"})
    class ActionPhaseStruct extends Structure {
        public Pointer phase;
        public Pointer body;
        public long bodyLen;

        public ActionPhaseStruct() {
            super();
        }

        public ActionPhaseStruct(Pointer p) {
            super(p);
            read();
        }
    }
}
