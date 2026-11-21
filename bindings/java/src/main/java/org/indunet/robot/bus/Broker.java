package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** In-process / local broker process handle. */
public final class Broker implements AutoCloseable {
    private Pointer ptr;

    public Broker() {
        this.ptr = start(null, null);
    }

    public Broker(BrokerOptions options) {
        this.ptr = start(null, options);
    }

    /** Start broker sharing {@code context} (required for same-process inproc Nodes). */
    public Broker(Context context) {
        this.ptr = start(context, null);
    }

    public Broker(Context context, BrokerOptions options) {
        this.ptr = start(context, options);
    }

    /**
     * When {@link AndroidRuntime#ipcRoot} is set, pass it through so Android brokers keep IPC
     * sockets under the app cache directory. Desktop starts keep the previous entry points.
     */
    private static Pointer start(Context context, BrokerOptions options) {
        Pointer ctx = context != null ? context.raw() : null;
        RobotBusC.BrokerOptions nativeOpts = options != null ? options.toNative() : null;
        String ipcRoot = AndroidRuntime.ipcRoot;
        if (ipcRoot != null) {
            return Errors.checkPtr(
                    RobotBusC.Holder.INSTANCE.robot_bus_broker_start_with_context_in(
                            ctx, nativeOpts, ipcRoot),
                    "Broker");
        }
        if (context == null) {
            return Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_broker_start(nativeOpts), "Broker");
        }
        return Errors.checkPtr(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_start_with_context(ctx, nativeOpts), "Broker");
    }

    public void stop() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_broker_stop(ptr), "broker stop");
    }

    public String messageXsubBind() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_message_xsub_bind(ptr));
    }

    public String messageXpubBind() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_message_xpub_bind(ptr));
    }

    public String serviceFrontendBind() {
        return NativeUtils.takeCString(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_service_frontend_bind(ptr));
    }

    public String serviceBackendBind() {
        return NativeUtils.takeCString(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_service_backend_bind(ptr));
    }

    public String actionFrontendBind() {
        return NativeUtils.takeCString(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_action_frontend_bind(ptr));
    }

    public String actionBackendBind() {
        return NativeUtils.takeCString(
                RobotBusC.Holder.INSTANCE.robot_bus_broker_action_backend_bind(ptr));
    }

    public String apiListen() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_api_listen(ptr));
    }

    public String consoleListen() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_console_listen(ptr));
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_broker_free(ptr);
        ptr = null;
    }
}
