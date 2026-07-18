package org.indunet.robot.bus;

/** Options for constructing a {@link Node} (maps to C {@code RobotBusNodeOptions}). */
public final class NodeOptions {
    private final String host;
    private final String transport;
    private final String grpcUrl;
    private final String messageXsub;
    private final String messageXpub;
    private final String serviceFrontend;
    private final String serviceBackend;
    private final String actionBackend;
    private final String actionFrontend;

    public NodeOptions() {
        this("localhost", "tcp", null, null, null, null, null, null, null);
    }

    public NodeOptions(
            String host,
            String transport,
            String grpcUrl,
            String messageXsub,
            String messageXpub,
            String serviceFrontend,
            String serviceBackend,
            String actionBackend,
            String actionFrontend) {
        this.host = host;
        this.transport = transport;
        this.grpcUrl = grpcUrl;
        this.messageXsub = messageXsub;
        this.messageXpub = messageXpub;
        this.serviceFrontend = serviceFrontend;
        this.serviceBackend = serviceBackend;
        this.actionBackend = actionBackend;
        this.actionFrontend = actionFrontend;
    }

    public String getHost() {
        return host;
    }

    public String getTransport() {
        return transport;
    }

    public String getGrpcUrl() {
        return grpcUrl;
    }

    public String getMessageXsub() {
        return messageXsub;
    }

    public String getMessageXpub() {
        return messageXpub;
    }

    public String getServiceFrontend() {
        return serviceFrontend;
    }

    public String getServiceBackend() {
        return serviceBackend;
    }

    public String getActionBackend() {
        return actionBackend;
    }

    public String getActionFrontend() {
        return actionFrontend;
    }

    RobotBusC.NodeOptions toNative() {
        RobotBusC.NodeOptions o = new RobotBusC.NodeOptions();
        o.host = host;
        o.transport = transport;
        o.grpcUrl = grpcUrl;
        o.messageXsub = messageXsub;
        o.messageXpub = messageXpub;
        o.serviceFrontend = serviceFrontend;
        o.serviceBackend = serviceBackend;
        o.actionBackend = actionBackend;
        o.actionFrontend = actionFrontend;
        o.write();
        return o;
    }
}
