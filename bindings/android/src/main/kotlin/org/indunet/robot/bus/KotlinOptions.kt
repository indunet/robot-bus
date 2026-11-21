package org.indunet.robot.bus

/**
 * Kotlin call-site helpers for the shared Java option types.
 *
 * Java constructors have no default values, and Kotlin cannot use named arguments on them.
 * These functions keep the previous Android call style (`tcpOnly = true`, `apiUrl = …`).
 */

fun brokerOptions(
    messageXsubBind: String? = null,
    messageXpubBind: String? = null,
    serviceFrontendBind: String? = null,
    serviceBackendBind: String? = null,
    actionFrontendBind: String? = null,
    actionBackendBind: String? = null,
    apiListen: String? = null,
    consoleListen: String? = null,
    tcpOnly: Boolean = false,
    noConsole: Boolean = false,
    brokerId: String? = null,
    messagePeers: List<String>? = null,
    servicePeers: List<String>? = null,
    actionPeers: List<String>? = null,
    noDiscovery: Boolean = false,
    domainId: Int = 0,
    advertiseHost: String? = null,
    peers: List<String>? = null,
    noTank: Boolean = false,
    noDocs: Boolean = false,
): BrokerOptions =
    BrokerOptions(
        messageXsubBind,
        messageXpubBind,
        serviceFrontendBind,
        serviceBackendBind,
        actionFrontendBind,
        actionBackendBind,
        apiListen,
        consoleListen,
        tcpOnly,
        noConsole,
        brokerId,
        messagePeers,
        servicePeers,
        actionPeers,
        noDiscovery,
        domainId,
        advertiseHost,
        peers,
        noTank,
        noDocs,
    )

fun nodeOptions(
    host: String? = "localhost",
    transport: String? = "tcp",
    wsUrl: String? = null,
    messageXsub: String? = null,
    messageXpub: String? = null,
    serviceFrontend: String? = null,
    serviceBackend: String? = null,
    actionBackend: String? = null,
    actionFrontend: String? = null,
): NodeOptions =
    NodeOptions(
        host,
        transport,
        wsUrl,
        messageXsub,
        messageXpub,
        serviceFrontend,
        serviceBackend,
        actionBackend,
        actionFrontend,
    )

fun discoverOpts(
    apiUrl: String? = null,
    brokerId: String? = null,
    timeoutSecs: Double = 0.0,
): DiscoverOpts = DiscoverOpts(apiUrl, brokerId, timeoutSecs)
