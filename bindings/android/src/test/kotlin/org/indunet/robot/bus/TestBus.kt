package org.indunet.robot.bus

import java.io.IOException
import java.net.ServerSocket

/** Ephemeral TCP broker + matching node options (mirrors Java / C++ TestBus). */
internal class TestBus private constructor(
    val broker: Broker,
    val messageXsub: String,
    val messageXpub: String,
    val serviceFrontend: String,
    val serviceBackend: String,
    val actionFrontend: String,
    val actionBackend: String,
    val grpcListen: String,
) : AutoCloseable {
    fun grpcUrl(): String = "http://$grpcListen"

    fun makeNode(name: String): Node =
        Node(
            name,
            NodeOptions(
                host = null,
                transport = "tcp",
                grpcUrl = null,
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
        @Throws(IOException::class)
        fun start(): TestBus {
            val messageXsub = "tcp://127.0.0.1:${freePort()}"
            val messageXpub = "tcp://127.0.0.1:${freePort()}"
            val serviceFrontend = "tcp://127.0.0.1:${freePort()}"
            val serviceBackend = "tcp://127.0.0.1:${freePort()}"
            val actionFrontend = "tcp://127.0.0.1:${freePort()}"
            val actionBackend = "tcp://127.0.0.1:${freePort()}"
            val grpcListen = "127.0.0.1:${freePort()}"
            val opts =
                BrokerOptions(
                    messageXsubBind = messageXsub,
                    messageXpubBind = messageXpub,
                    serviceFrontendBind = serviceFrontend,
                    serviceBackendBind = serviceBackend,
                    actionFrontendBind = actionFrontend,
                    actionBackendBind = actionBackend,
                    grpcListen = grpcListen,
                    consoleListen = null,
                    tcpOnly = true,
                    noConsole = true,
                )
            return TestBus(
                Broker(opts),
                messageXsub,
                messageXpub,
                serviceFrontend,
                serviceBackend,
                actionFrontend,
                actionBackend,
                grpcListen,
            )
        }

        @Throws(IOException::class)
        private fun freePort(): Int =
            ServerSocket(0).use { socket ->
                socket.reuseAddress = true
                socket.localPort
            }
    }
}
