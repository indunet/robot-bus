package org.indunet.robot.bus;

import static org.indunet.robot.bus.Endpoints.messageXpubEndpoint;
import static org.indunet.robot.bus.Endpoints.messageXsubEndpoint;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class EndpointSmokeTest {
    @Test
    void messageEndpointsResolve() {
        String xsub = messageXsubEndpoint("127.0.0.1", "tcp");
        String xpub = messageXpubEndpoint("127.0.0.1", "tcp");
        assertTrue(xsub.contains("127.0.0.1"), "xsub=" + xsub);
        assertTrue(xpub.contains("127.0.0.1"), "xpub=" + xpub);
    }
}
