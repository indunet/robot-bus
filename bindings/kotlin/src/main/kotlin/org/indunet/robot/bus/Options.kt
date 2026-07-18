package org.indunet.robot.bus

/** Options for constructing a [Node] (maps to C `RobotBusNodeOptions`). */
data class NodeOptions(
    val host: String? = "localhost",
    val transport: String? = "tcp",
    val grpcUrl: String? = null,
    val messageXsub: String? = null,
    val messageXpub: String? = null,
    val serviceFrontend: String? = null,
    val serviceBackend: String? = null,
    val actionBackend: String? = null,
    val actionFrontend: String? = null,
) {
    internal fun toNative(): RobotBusC.NodeOptions {
        val o = RobotBusC.NodeOptions()
        o.host = host
        o.transport = transport
        o.grpcUrl = grpcUrl
        o.messageXsub = messageXsub
        o.messageXpub = messageXpub
        o.serviceFrontend = serviceFrontend
        o.serviceBackend = serviceBackend
        o.actionBackend = actionBackend
        o.actionFrontend = actionFrontend
        o.write()
        return o
    }
}

/** Options for starting a [Broker] (maps to C `RobotBusBrokerOptions`). */
data class BrokerOptions(
    val messageXsubBind: String? = null,
    val messageXpubBind: String? = null,
    val serviceFrontendBind: String? = null,
    val serviceBackendBind: String? = null,
    val actionFrontendBind: String? = null,
    val actionBackendBind: String? = null,
    val grpcListen: String? = null,
    val consoleListen: String? = null,
    val tcpOnly: Boolean = false,
    val noConsole: Boolean = false,
) {
    internal fun toNative(): RobotBusC.BrokerOptions {
        val o = RobotBusC.BrokerOptions()
        o.messageXsubBind = messageXsubBind
        o.messageXpubBind = messageXpubBind
        o.serviceFrontendBind = serviceFrontendBind
        o.serviceBackendBind = serviceBackendBind
        o.actionFrontendBind = actionFrontendBind
        o.actionBackendBind = actionBackendBind
        o.grpcListen = grpcListen
        o.consoleListen = consoleListen
        o.tcpOnly = if (tcpOnly) 1 else 0
        o.noConsole = if (noConsole) 1 else 0
        o.write()
        return o
    }
}
