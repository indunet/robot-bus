package org.indunet.robot.bus;

import com.sun.jna.StringArray;
import java.util.Collections;
import java.util.List;

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
    private final String brokerId;
    private final List<String> messagePeers;
    private final List<String> servicePeers;
    private final List<String> actionPeers;

    /** Keep-alive for native {@code char**} peer arrays until {@link Broker} start returns. */
    private transient StringArray messagePeersNative;
    private transient StringArray servicePeersNative;
    private transient StringArray actionPeersNative;

    public BrokerOptions() {
        this(null, null, null, null, null, null, null, null, false, false, null, null, null, null);
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
        this(
                messageXsubBind,
                messageXpubBind,
                serviceFrontendBind,
                serviceBackendBind,
                actionFrontendBind,
                actionBackendBind,
                grpcListen,
                consoleListen,
                tcpOnly,
                noConsole,
                null,
                null,
                null,
                null);
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
            boolean noConsole,
            String brokerId,
            List<String> messagePeers,
            List<String> servicePeers,
            List<String> actionPeers) {
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
        this.brokerId = brokerId;
        this.messagePeers = messagePeers;
        this.servicePeers = servicePeers;
        this.actionPeers = actionPeers;
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

    public String getBrokerId() {
        return brokerId;
    }

    public List<String> getMessagePeers() {
        return messagePeers == null ? Collections.emptyList() : messagePeers;
    }

    public List<String> getServicePeers() {
        return servicePeers == null ? Collections.emptyList() : servicePeers;
    }

    public List<String> getActionPeers() {
        return actionPeers == null ? Collections.emptyList() : actionPeers;
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
        o.brokerId = brokerId;
        if (messagePeers != null && !messagePeers.isEmpty()) {
            messagePeersNative = new StringArray(messagePeers.toArray(new String[0]));
            o.messagePeers = messagePeersNative;
            o.messagePeerCount = messagePeers.size();
        }
        if (servicePeers != null && !servicePeers.isEmpty()) {
            servicePeersNative = new StringArray(servicePeers.toArray(new String[0]));
            o.servicePeers = servicePeersNative;
            o.servicePeerCount = servicePeers.size();
        }
        if (actionPeers != null && !actionPeers.isEmpty()) {
            actionPeersNative = new StringArray(actionPeers.toArray(new String[0]));
            o.actionPeers = actionPeersNative;
            o.actionPeerCount = actionPeers.size();
        }
        o.write();
        return o;
    }
}
