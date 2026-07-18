package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** In-process / local broker process handle. */
public final class Broker implements AutoCloseable {
    private Pointer ptr;

    public Broker() {
        this.ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_broker_start(null), "Broker");
    }

    public Broker(BrokerOptions options) {
        this.ptr =
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_broker_start(options.toNative()), "Broker");
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

    public String grpcListen() {
        return NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_broker_grpc_listen(ptr));
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
