package org.indunet.robot.bus;

/** Options for constructing a {@link Node} (maps to C {@code RobotBusNodeOptions}). */
public final class NodeOptions {
    private final String host;
    private final String transport;
    private final String wsUrl;
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
            String wsUrl,
            String messageXsub,
            String messageXpub,
            String serviceFrontend,
            String serviceBackend,
            String actionBackend,
            String actionFrontend) {
        this.host = host;
        this.transport = transport;
        this.wsUrl = wsUrl;
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
        return wsUrl;
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

    /** Discover broker endpoints for {@code transport} and return filled options. */
    public static NodeOptions discover(String transport, DiscoverOpts opts) {
        RobotBusC.AppliedNodeOptions out = new RobotBusC.AppliedNodeOptions();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_discover_node_options(
                        transport, opts != null ? opts.toNative() : null, out),
                "NodeOptions.discover");
        out.read();
        try {
            return new NodeOptions(
                    ptrString(out.host) != null ? ptrString(out.host) : "localhost",
                    ptrString(out.transport) != null ? ptrString(out.transport) : transport,
                    ptrString(out.wsUrl),
                    ptrString(out.messageXsub),
                    ptrString(out.messageXpub),
                    ptrString(out.serviceFrontend),
                    ptrString(out.serviceBackend),
                    ptrString(out.actionBackend),
                    ptrString(out.actionFrontend));
        } finally {
            RobotBusC.Holder.INSTANCE.robot_bus_applied_node_options_free(out);
        }
    }

    private static String ptrString(com.sun.jna.Pointer p) {
        return p != null ? p.getString(0) : null;
    }

    RobotBusC.NodeOptions toNative() {
        RobotBusC.NodeOptions o = new RobotBusC.NodeOptions();
        o.host = host;
        o.transport = transport;
        o.wsUrl = wsUrl;
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
