package org.indunet.robot.bus;

/** Options for starting a {@link Broker} (maps to C {@code RobotBusBrokerOptions}). */
public final class BrokerOptions {
    private final String messageXsubBind;
    private final String messageXpubBind;
    private final String serviceFrontendBind;
    private final String serviceBackendBind;
    private final String actionFrontendBind;
    private final String actionBackendBind;
    private final String grpcListen;
    private final String consoleListen;
    private final boolean tcpOnly;
    private final boolean noConsole;

    public BrokerOptions() {
        this(null, null, null, null, null, null, null, null, false, false);
    }

    public BrokerOptions(
            String messageXsubBind,
            String messageXpubBind,
            String serviceFrontendBind,
            String serviceBackendBind,
            String actionFrontendBind,
            String actionBackendBind,
            String grpcListen,
            String consoleListen,
            boolean tcpOnly,
            boolean noConsole) {
        this.messageXsubBind = messageXsubBind;
        this.messageXpubBind = messageXpubBind;
        this.serviceFrontendBind = serviceFrontendBind;
        this.serviceBackendBind = serviceBackendBind;
        this.actionFrontendBind = actionFrontendBind;
        this.actionBackendBind = actionBackendBind;
        this.grpcListen = grpcListen;
        this.consoleListen = consoleListen;
        this.tcpOnly = tcpOnly;
        this.noConsole = noConsole;
    }

    public String getMessageXsubBind() {
        return messageXsubBind;
    }

    public String getMessageXpubBind() {
        return messageXpubBind;
    }

    public String getServiceFrontendBind() {
        return serviceFrontendBind;
    }

    public String getServiceBackendBind() {
        return serviceBackendBind;
    }

    public String getActionFrontendBind() {
        return actionFrontendBind;
    }

    public String getActionBackendBind() {
        return actionBackendBind;
    }

    public String getGrpcListen() {
        return grpcListen;
    }

    public String getConsoleListen() {
        return consoleListen;
    }

    public boolean isTcpOnly() {
        return tcpOnly;
    }

    public boolean isNoConsole() {
        return noConsole;
    }

    RobotBusC.BrokerOptions toNative() {
        RobotBusC.BrokerOptions o = new RobotBusC.BrokerOptions();
        o.messageXsubBind = messageXsubBind;
        o.messageXpubBind = messageXpubBind;
        o.serviceFrontendBind = serviceFrontendBind;
        o.serviceBackendBind = serviceBackendBind;
        o.actionFrontendBind = actionFrontendBind;
        o.actionBackendBind = actionBackendBind;
        o.grpcListen = grpcListen;
        o.consoleListen = consoleListen;
        o.tcpOnly = tcpOnly ? 1 : 0;
        o.noConsole = noConsole ? 1 : 0;
        o.write();
        return o;
    }
}
