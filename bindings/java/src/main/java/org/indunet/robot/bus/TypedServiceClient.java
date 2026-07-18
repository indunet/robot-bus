package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;

/** Service client that encodes requests and decodes responses. */
public final class TypedServiceClient<Req extends MessageLite, Resp extends MessageLite>
        implements AutoCloseable {
    private final ServiceClient inner;
    private final Class<Req> requestType;
    private final Class<Resp> responseType;

    TypedServiceClient(ServiceClient inner, Class<Req> requestType, Class<Resp> responseType) {
        this.inner = inner;
        this.requestType = requestType;
        this.responseType = responseType;
    }

    public String serviceName() {
        return inner.serviceName();
    }

    public Class<Req> requestType() {
        return requestType;
    }

    public Class<Resp> responseType() {
        return responseType;
    }

    public Resp call(Req request) {
        return call(request, -1.0);
    }

    public Resp call(Req request, double timeoutSecs) {
        if (request == null) {
            throw new NullPointerException("request");
        }
        if (!requestType.isInstance(request)) {
            throw new IllegalArgumentException(
                    "client for "
                            + requestType.getSimpleName()
                            + " got "
                            + request.getClass().getSimpleName());
        }
        byte[] raw = inner.call(ProtoCodec.encode(request), timeoutSecs);
        Resp reply = ProtoCodec.tryParse(responseType, raw);
        if (reply == null) {
            throw new IllegalStateException(
                    "service " + serviceName() + " response decode failed for " + responseType.getSimpleName());
        }
        return reply;
    }

    @Override
    public void close() {
        inner.close();
    }

    @Override
    public String toString() {
        return "TypedServiceClient{service="
                + serviceName()
                + ", request="
                + requestType.getSimpleName()
                + ", response="
                + responseType.getSimpleName()
                + '}';
    }
}
