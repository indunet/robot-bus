package org.indunet.robot.bus

/** Options for constructing a [Node] (maps to C `RobotBusNodeOptions`). */
class NodeOptions
@JvmOverloads
constructor(
    private val host: String? = "localhost",
    private val transport: String? = "tcp",
    private val grpcUrl: String? = null,
    private val messageXsub: String? = null,
    private val messageXpub: String? = null,
    private val serviceFrontend: String? = null,
    private val serviceBackend: String? = null,
    private val actionBackend: String? = null,
    private val actionFrontend: String? = null,
) {
    fun getHost(): String? = host

    fun getTransport(): String? = transport

    fun getGrpcUrl(): String? = grpcUrl

    fun getMessageXsub(): String? = messageXsub

    fun getMessageXpub(): String? = messageXpub

    fun getServiceFrontend(): String? = serviceFrontend

    fun getServiceBackend(): String? = serviceBackend

    fun getActionBackend(): String? = actionBackend

    fun getActionFrontend(): String? = actionFrontend

    companion object {
        /** Discover broker endpoints for [transport] and return filled options. */
        @JvmStatic
        @JvmOverloads
        fun discover(transport: String, opts: DiscoverOpts? = null): NodeOptions {
            val out = RobotBusC.AppliedNodeOptions()
            Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_discover_node_options(
                    transport,
                    opts?.toNative(),
                    out,
                ),
                "NodeOptions.discover",
            )
            out.read()
            try {
                fun ptrString(p: com.sun.jna.Pointer?): String? = p?.getString(0)
                return NodeOptions(
                    host = ptrString(out.host) ?: "localhost",
                    transport = ptrString(out.transport) ?: transport,
                    grpcUrl = ptrString(out.grpcUrl),
                    messageXsub = ptrString(out.messageXsub),
                    messageXpub = ptrString(out.messageXpub),
                    serviceFrontend = ptrString(out.serviceFrontend),
                    serviceBackend = ptrString(out.serviceBackend),
                    actionBackend = ptrString(out.actionBackend),
                    actionFrontend = ptrString(out.actionFrontend),
                )
            } finally {
                RobotBusC.Holder.INSTANCE.robot_bus_applied_node_options_free(out)
            }
        }
    }

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
