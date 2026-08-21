package org.indunet.robot.bus

/** Ephemeral TCP broker + matching node options (mirrors Java / C++ TestBus). */
internal class TestBus private constructor(
    val broker: Broker,
    val messageXsub: String,
    val messageXpub: String,
    val serviceFrontend: String,
    val serviceBackend: String,
    val actionFrontend: String,
    val actionBackend: String,
    val apiListen: String,
) : AutoCloseable {
    fun wsUrl(): String = "http://$apiListen"

    fun makeNode(name: String): Node =
        Node(
            name,
            NodeOptions(
                host = null,
                transport = "tcp",
                wsUrl = null,
                messageXsub = messageXsub,
                messageXpub = messageXpub,
                serviceFrontend = serviceFrontend,
                serviceBackend = serviceBackend,
                actionBackend = actionBackend,
                actionFrontend = actionFrontend,
            ),
        )

    override fun close() {
        broker.stop()
        broker.close()
    }

    companion object {
        @JvmStatic
        fun start(): TestBus {
            // Bind :0 so the OS assigns ports at broker start (avoids freePort TOCTOU).
            val opts =
                BrokerOptions(
                    messageXsubBind = "tcp://127.0.0.1:0",
                    messageXpubBind = "tcp://127.0.0.1:0",
                    serviceFrontendBind = "tcp://127.0.0.1:0",
                    serviceBackendBind = "tcp://127.0.0.1:0",
                    actionFrontendBind = "tcp://127.0.0.1:0",
                    actionBackendBind = "tcp://127.0.0.1:0",
                    apiListen = "127.0.0.1:0",
                    consoleListen = null,
                    tcpOnly = true,
                    noConsole = true,
                )
            val broker = Broker(opts)
            return TestBus(
                broker,
                broker.messageXsubBind(),
                broker.messageXpubBind(),
                broker.serviceFrontendBind(),
                broker.serviceBackendBind(),
                broker.actionFrontendBind(),
                broker.actionBackendBind(),
                broker.apiListen(),
            )
        }
    }
}
