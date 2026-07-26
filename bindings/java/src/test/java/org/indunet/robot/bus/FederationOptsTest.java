package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.net.ServerSocket;
import java.util.List;
import org.junit.jupiter.api.Test;

/** Smoke: broker start accepts federation peer options (CLI-compatible strings). */
class FederationOptsTest {
    @Test
    void startWithFederationPeers() throws Exception {
        String peerXpub = "tcp://127.0.0.1:" + freePort();
        String peerSvc = "tcp://127.0.0.1:" + freePort();
        String peerAct = "tcp://127.0.0.1:" + freePort();
        BrokerOptions opts =
                new BrokerOptions(
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "127.0.0.1:" + freePort(),
                        null,
                        true,
                        true,
                        "broker-a",
                        List.of(peerXpub),
                        List.of("broker-b=" + peerSvc),
                        List.of("broker-b=" + peerAct));
        try (Broker broker = new Broker(opts)) {
            assertTrue(broker.messageXsubBind().startsWith("tcp://"));
        }
    }

    @Test
    void startRejectsInvalidMessagePeer() throws Exception {
        BrokerOptions opts =
                new BrokerOptions(
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "tcp://127.0.0.1:" + freePort(),
                        "127.0.0.1:" + freePort(),
                        null,
                        true,
                        true,
                        null,
                        List.of("tcp://127.0.0.1:0"),
                        null,
                        null);
        RobotBusException err = assertThrows(RobotBusException.class, () -> new Broker(opts));
        assertTrue(err.getMessage().contains("invalid message peer"), err.getMessage());
    }

    private static int freePort() throws IOException {
        try (ServerSocket socket = new ServerSocket(0)) {
            socket.setReuseAddress(true);
            return socket.getLocalPort();
        }
    }
}
