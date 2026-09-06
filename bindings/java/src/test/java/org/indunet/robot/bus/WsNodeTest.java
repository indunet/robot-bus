package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;

/**
 * WebSocket-mode Node tests aligned with Python {@code test_ws_node.py} /
 * TypeScript {@code ws_node.test.ts}.
 */
class WsNodeTest {
    @Test
    void grpcConstructors() {
        try (Node node = Node.ws("web")) {
            assertEquals("web", node.name());
        }
        try (Node node = Node.wsAt("web2", "http://10.0.0.1:15560")) {
            assertEquals("web2", node.name());
        }
        try (Node node =
                new Node(
                        "web3",
                        new NodeOptions(
                                null,
                                "ws",
                                "http://127.0.0.1:15560",
                                null,
                                null,
                                null,
                                null,
                                null,
                                null))) {
            assertEquals("web3", node.name());
        }
    }

    @Test
    void grpcNodeRejectsServers() {
        try (Node node = Node.ws("only-client")) {
            RobotBusException svc =
                    assertThrows(
                            RobotBusException.class,
                            () -> node.createService("/svc", body -> new byte[0]));
            assertTrue(svc.getMessage().toLowerCase().contains("not supported"));

            RobotBusException act =
                    assertThrows(
                            RobotBusException.class,
                            () -> node.createActionServer("/act", body -> List.of()));
            assertTrue(act.getMessage().toLowerCase().contains("not supported"));
        }
    }

    @Test
    void grpcNodePublish() throws Exception {
        try (TestBus bus = TestBus.start();
                Subscriber sub = new Subscriber(bus.messageXpub);
                Node client = Node.wsAt("grpc_pub", bus.wsUrl())) {
            sub.subscribe("java.ws.pub");
            Thread.sleep(200);

            TopicPublisher pub = client.createPublisher("java.ws.pub");
            pub.publish("from-java-grpc".getBytes(StandardCharsets.UTF_8));

            TopicMessage msg = sub.receive(3.0);
            assertEquals("java.ws.pub", msg.getTopic());
            assertEquals("from-java-grpc", new String(msg.getPayload(), StandardCharsets.UTF_8));
        }
    }

    @Test
    void grpcNodeSubscribeAndService() throws Exception {
        try (TestBus bus = TestBus.start();
                Publisher pub = new Publisher(bus.messageXsub);
                Node server = bus.makeNode("svc_server");
                Node client = Node.wsAt("grpc_client", bus.wsUrl())) {
            server.createService("svc.java_grpc_echo", body -> {
                byte[] prefix = "echo:".getBytes(StandardCharsets.UTF_8);
                byte[] out = new byte[prefix.length + body.length];
                System.arraycopy(prefix, 0, out, 0, prefix.length);
                System.arraycopy(body, 0, out, prefix.length, body.length);
                return out;
            });
            server.start();
            Thread.sleep(200);

            AtomicReference<byte[]> gotPayload = new AtomicReference<>();
            client.createSubscription(
                    "java.ws.topic",
                    (payload) -> {
                        gotPayload.set(payload);
                    });
            Thread.sleep(300);

            pub.publish("java.ws.topic", "hello-java-grpc".getBytes(StandardCharsets.UTF_8));
            assertTrue(
                    waitUntil(
                            () -> {
                                client.spinOnce(0.05);
                                return gotPayload.get() != null;
                            },
                            5000));
            assertEquals(
                    "hello-java-grpc", new String(gotPayload.get(), StandardCharsets.UTF_8));

            ServiceClient svc = client.createClient("svc.java_grpc_echo");
            byte[] reply = svc.call("ping".getBytes(StandardCharsets.UTF_8), 3.0);
            assertEquals("echo:ping", new String(reply, StandardCharsets.UTF_8));

            server.shutdown();
            server.stop();
            server.waitForShutdown();
        }
    }

    @Test
    void grpcNodeActionClient() throws Exception {
        try (TestBus bus = TestBus.start();
                Node server = bus.makeNode("act_server");
                Node client = Node.wsAt("grpc_action", bus.wsUrl())) {
            server.createActionServer(
                    "act.java_grpc_demo",
                    body -> {
                        List<ActionPhase> phases = new ArrayList<>();
                        phases.add(new ActionPhase("FEEDBACK", "step-1".getBytes(StandardCharsets.UTF_8)));
                        phases.add(new ActionPhase("FEEDBACK", "step-2".getBytes(StandardCharsets.UTF_8)));
                        byte[] result = new byte["done:".length() + body.length];
                        byte[] prefix = "done:".getBytes(StandardCharsets.UTF_8);
                        System.arraycopy(prefix, 0, result, 0, prefix.length);
                        System.arraycopy(body, 0, result, prefix.length, body.length);
                        phases.add(new ActionPhase("RESULT", result));
                        return phases;
                    });
            server.start();
            Thread.sleep(200);

            ActionClient action = client.createActionClient("act.java_grpc_demo");
            List<ActionMessage> feedback = new ArrayList<>();
            try (ActionGoalHandle goal =
                    action.sendGoal(
                            "fly".getBytes(StandardCharsets.UTF_8), null, 5.0, feedback::add)) {
                ActionMessage result = goal.result(5.0);
                assertEquals(2, feedback.size());
                assertEquals("FEEDBACK", feedback.get(0).getKind());
                assertEquals(
                        "step-1",
                        new String(feedback.get(0).getBody(), StandardCharsets.UTF_8));
                assertEquals("RESULT", result.getKind());
                assertEquals("done:fly", new String(result.getBody(), StandardCharsets.UTF_8));
            }

            server.shutdown();
            server.stop();
            server.waitForShutdown();
        }
    }

    private static boolean waitUntil(Check check, long timeoutMs) throws Exception {
        long deadline = System.currentTimeMillis() + timeoutMs;
        while (System.currentTimeMillis() < deadline) {
            if (check.ok()) {
                return true;
            }
            Thread.sleep(20);
        }
        return check.ok();
    }

    @FunctionalInterface
    private interface Check {
        boolean ok() throws Exception;
    }
}
