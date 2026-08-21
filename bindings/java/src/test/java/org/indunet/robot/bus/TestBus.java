package org.indunet.robot.bus;

/** Ephemeral TCP broker + matching node options (mirrors C++ {@code TestBus}). */
final class TestBus implements AutoCloseable {
    final Broker broker;
    final String messageXsub;
    final String messageXpub;
    final String serviceFrontend;
    final String serviceBackend;
    final String actionFrontend;
    final String actionBackend;
    final String apiListen;

    private TestBus(
            Broker broker,
            String messageXsub,
            String messageXpub,
            String serviceFrontend,
            String serviceBackend,
            String actionFrontend,
            String actionBackend,
            String apiListen) {
        this.broker = broker;
        this.messageXsub = messageXsub;
        this.messageXpub = messageXpub;
        this.serviceFrontend = serviceFrontend;
        this.serviceBackend = serviceBackend;
        this.actionFrontend = actionFrontend;
        this.actionBackend = actionBackend;
        this.apiListen = apiListen;
    }

    static TestBus start() {
        // Bind :0 so the OS assigns ports at broker start (avoids freePort TOCTOU).
        BrokerOptions opts =
                new BrokerOptions(
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "127.0.0.1:0",
                        null,
                        true,
                        true);
        Broker broker = new Broker(opts);
        return new TestBus(
                broker,
                broker.messageXsubBind(),
                broker.messageXpubBind(),
                broker.serviceFrontendBind(),
                broker.serviceBackendBind(),
                broker.actionFrontendBind(),
                broker.actionBackendBind(),
                broker.apiListen());
    }

    String wsUrl() {
        return "http://" + apiListen;
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
}
