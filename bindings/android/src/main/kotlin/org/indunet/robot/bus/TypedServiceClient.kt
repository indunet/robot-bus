package org.indunet.robot.bus

import com.google.protobuf.MessageLite

/** Service client that encodes requests and decodes responses. */
class TypedServiceClient<Req : MessageLite, Resp : MessageLite> internal constructor(
    private val inner: ServiceClient,
    private val requestType: Class<Req>,
    private val responseType: Class<Resp>,
) : AutoCloseable {
    fun serviceName(): String = inner.serviceName()

    fun requestType(): Class<Req> = requestType

    fun responseType(): Class<Resp> = responseType

    @JvmOverloads
    fun call(request: Req, timeoutSecs: Double = -1.0): Resp {
        requireNotNull(request) { "request" }
        if (!requestType.isInstance(request)) {
            throw IllegalArgumentException(
                "client for ${requestType.simpleName} got ${request.javaClass.simpleName}",
            )
        }
        val raw = inner.call(ProtoCodec.encode(request), timeoutSecs)
        return ProtoCodec.tryParse(responseType, raw)
            ?: throw IllegalStateException(
                "service ${serviceName()} response decode failed for ${responseType.simpleName}",
            )
    }

    override fun close() {
        inner.close()
    }

    override fun toString(): String =
        "TypedServiceClient{service=${serviceName()}, request=${requestType.simpleName}, response=${responseType.simpleName}}"
}
