package org.indunet.robot.bus

import kotlin.test.Test
import kotlin.test.assertTrue

class EndpointSmokeTest {
    @Test
    fun messageEndpointsResolve() {
        val xsub = messageXsubEndpoint("127.0.0.1", "tcp")
        val xpub = messageXpubEndpoint("127.0.0.1", "tcp")
        assertTrue(xsub.contains("127.0.0.1"), "xsub=$xsub")
        assertTrue(xpub.contains("127.0.0.1"), "xpub=$xpub")
    }
}
