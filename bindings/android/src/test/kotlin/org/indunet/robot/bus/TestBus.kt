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
        private const val PORT_COUNT = 7
        private const val MAX_START_ATTEMPTS = 5

        @JvmStatic
        @Throws(IOException::class)
        fun start(): TestBus {
            repeat(MAX_START_ATTEMPTS) { attempt ->
                // Reserve the complete set together so one TestBus never receives duplicate ports.
                // The sockets must be released before the native broker binds, so retry the small
                // remaining race with other processes.
                val ports = reservePorts(PORT_COUNT)
                val messageXsub = "tcp://127.0.0.1:${ports[0]}"
                val messageXpub = "tcp://127.0.0.1:${ports[1]}"
                val serviceFrontend = "tcp://127.0.0.1:${ports[2]}"
                val serviceBackend = "tcp://127.0.0.1:${ports[3]}"
                val actionFrontend = "tcp://127.0.0.1:${ports[4]}"
                val actionBackend = "tcp://127.0.0.1:${ports[5]}"
                val apiListen = "127.0.0.1:${ports[6]}"
                val opts =
                    BrokerOptions(
                        messageXsubBind = messageXsub,
                        messageXpubBind = messageXpub,
                        serviceFrontendBind = serviceFrontend,
                        serviceBackendBind = serviceBackend,
                        actionFrontendBind = actionFrontend,
                        actionBackendBind = actionBackend,
                        apiListen = apiListen,
                        consoleListen = null,
                        tcpOnly = true,
                        noConsole = true,
                    )
                try {
                    return TestBus(
                        Broker(opts),
                        messageXsub,
                        messageXpub,
                        serviceFrontend,
                        serviceBackend,
                        actionFrontend,
                        actionBackend,
                        apiListen,
                    )
                } catch (failure: RobotBusException) {
                    if (attempt == MAX_START_ATTEMPTS - 1) throw failure
                }
            }
            error("unreachable")
        }

        @Throws(IOException::class)
        private fun reservePorts(count: Int): List<Int> {
            val sockets = mutableListOf<ServerSocket>()
            return try {
                repeat(count) {
                    sockets += ServerSocket(0).apply {
                        reuseAddress = true
                    }
                }
                sockets.map { it.localPort }
            } finally {
                sockets.forEach { it.close() }
            }
        }
    }
}
