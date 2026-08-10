package org.indunet.robot.bus;

import java.io.IOException;
import java.net.ServerSocket;

/** Ephemeral TCP broker + matching node options (mirrors C++ {@code TestBus}). */
final class TestBus implements AutoCloseable {
    final Broker broker;
    final String messageXsub;
    final String messageXpub;
    final String serviceFrontend;
    final String serviceBackend;
    final String actionFrontend;
    final String actionBackend;
    final String grpcListen;

    private TestBus(
            Broker broker,
            String messageXsub,
            String messageXpub,
            String serviceFrontend,
            String serviceBackend,
            String actionFrontend,
            String actionBackend,
            String grpcListen) {
        this.broker = broker;
        this.messageXsub = messageXsub;
        this.messageXpub = messageXpub;
        this.serviceFrontend = serviceFrontend;
        this.serviceBackend = serviceBackend;
        this.actionFrontend = actionFrontend;
        this.actionBackend = actionBackend;
        this.grpcListen = grpcListen;
    }

    static TestBus start() throws IOException {
        String messageXsub = "tcp://127.0.0.1:" + freePort();
        String messageXpub = "tcp://127.0.0.1:" + freePort();
        String serviceFrontend = "tcp://127.0.0.1:" + freePort();
        String serviceBackend = "tcp://127.0.0.1:" + freePort();
        String actionFrontend = "tcp://127.0.0.1:" + freePort();
        String actionBackend = "tcp://127.0.0.1:" + freePort();
        String grpcListen = "127.0.0.1:" + freePort();
        BrokerOptions opts =
                new BrokerOptions(
                        messageXsub,
                        messageXpub,
                        serviceFrontend,
                        serviceBackend,
                        actionFrontend,
                        actionBackend,
                        grpcListen,
                        null,
                        true,
                        true);
        return new TestBus(
                new Broker(opts),
                messageXsub,
                messageXpub,
                serviceFrontend,
                serviceBackend,
                actionFrontend,
                actionBackend,
                grpcListen);
    }

    String wsUrl() {
        return "http://" + grpcListen;
    }

    Node makeNode(String name) {
        return new Node(
                name,
                new NodeOptions(
                        null,
                        "tcp",
                        null,
                        messageXsub,
                        messageXpub,
                        serviceFrontend,
                        serviceBackend,
                        actionBackend,
                        actionFrontend));
    }

    @Override
    public void close() {
        broker.stop();
        broker.close();
    }

    private static int freePort() throws IOException {
        try (ServerSocket socket = new ServerSocket(0)) {
            socket.setReuseAddress(true);
            return socket.getLocalPort();
        }
    }
}
