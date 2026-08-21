package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

/** Smoke: broker start accepts federation peer options (CLI-compatible strings). */
class FederationOptsTest {
    @Test
    void startWithFederationPeers() {
        BrokerOptions opts =
                new BrokerOptions(
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "127.0.0.1:0",
                        null,
                        true,
                        true,
                        "broker-a",
                        List.of("tcp://127.0.0.1:16561"),
                        List.of("broker-b=tcp://127.0.0.1:16562"),
                        List.of("broker-b=tcp://127.0.0.1:16563"));
        try (Broker broker = new Broker(opts)) {
            assertTrue(broker.messageXsubBind().startsWith("tcp://"));
        }
    }

    @Test
    void startRejectsInvalidMessagePeer() {
        BrokerOptions opts =
                new BrokerOptions(
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "tcp://127.0.0.1:0",
                        "127.0.0.1:0",
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
}
