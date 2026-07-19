package org.indunet.robot.bus

/** Options for starting a [Broker] (maps to C `RobotBusBrokerOptions`). */
class BrokerOptions
@JvmOverloads
constructor(
    private val messageXsubBind: String? = null,
    private val messageXpubBind: String? = null,
    private val serviceFrontendBind: String? = null,
    private val serviceBackendBind: String? = null,
    private val actionFrontendBind: String? = null,
    private val actionBackendBind: String? = null,
    private val grpcListen: String? = null,
    private val consoleListen: String? = null,
    private val tcpOnly: Boolean = false,
    private val noConsole: Boolean = false,
) {
    fun getMessageXsubBind(): String? = messageXsubBind

    fun getMessageXpubBind(): String? = messageXpubBind

    fun getServiceFrontendBind(): String? = serviceFrontendBind

    fun getServiceBackendBind(): String? = serviceBackendBind

    fun getActionFrontendBind(): String? = actionFrontendBind

    fun getActionBackendBind(): String? = actionBackendBind

    fun getGrpcListen(): String? = grpcListen

    fun getConsoleListen(): String? = consoleListen

    fun isTcpOnly(): Boolean = tcpOnly

    fun isNoConsole(): Boolean = noConsole

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
