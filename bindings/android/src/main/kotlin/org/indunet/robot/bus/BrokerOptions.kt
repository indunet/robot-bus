package org.indunet.robot.bus

import com.sun.jna.StringArray

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
    private val brokerId: String? = null,
    private val messagePeers: List<String>? = null,
    private val servicePeers: List<String>? = null,
    private val actionPeers: List<String>? = null,
) {
    /** Keep-alive for native `char**` peer arrays until [Broker] start returns. */
    private var messagePeersNative: StringArray? = null
    private var servicePeersNative: StringArray? = null
    private var actionPeersNative: StringArray? = null

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

    fun getBrokerId(): String? = brokerId

    fun getMessagePeers(): List<String> = messagePeers.orEmpty()

    fun getServicePeers(): List<String> = servicePeers.orEmpty()

    fun getActionPeers(): List<String> = actionPeers.orEmpty()

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
        o.brokerId = brokerId
        if (!messagePeers.isNullOrEmpty()) {
            messagePeersNative = StringArray(messagePeers.toTypedArray())
            o.messagePeers = messagePeersNative
            o.messagePeerCount = messagePeers.size.toLong()
        }
        if (!servicePeers.isNullOrEmpty()) {
            servicePeersNative = StringArray(servicePeers.toTypedArray())
            o.servicePeers = servicePeersNative
            o.servicePeerCount = servicePeers.size.toLong()
        }
        if (!actionPeers.isNullOrEmpty()) {
            actionPeersNative = StringArray(actionPeers.toTypedArray())
            o.actionPeers = actionPeersNative
            o.actionPeerCount = actionPeers.size.toLong()
        }
        o.write()
        return o
    }
}
