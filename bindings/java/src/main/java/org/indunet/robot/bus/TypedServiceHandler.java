package org.indunet.robot.bus;

import com.google.protobuf.MessageLite;

/** Handler for typed service requests. */
@FunctionalInterface
public interface TypedServiceHandler<Req extends MessageLite, Resp extends MessageLite> {
    Resp handle(Req request);
}
