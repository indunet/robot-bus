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
 * gRPC-mode Node tests aligned with Python {@code test_grpc_node.py} /
 * TypeScript {@code grpc_node.test.ts}.
 */
class GrpcNodeTest {
    @Test
    void grpcConstructors() {
        try (Node node = Node.grpc("web")) {
            assertEquals("web", node.name());
        }
        try (Node node = Node.grpcAt("web2", "http://10.0.0.1:15770")) {
            assertEquals("web2", node.name());
        }
        try (Node node =
                new Node(
                        "web3",
                        new NodeOptions(
                                null,
                                "grpc",
                                "http://127.0.0.1:15770",
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
        try (Node node = Node.grpc("only-client")) {
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
                Node client = Node.grpcAt("grpc_pub", bus.grpcUrl())) {
            sub.subscribe("java.grpc.pub");
            Thread.sleep(200);

            TopicPublisher pub = client.createPublisher("java.grpc.pub");
            pub.publish("from-java-grpc".getBytes(StandardCharsets.UTF_8));

            TopicMessage msg = sub.receive(3.0);
            assertEquals("java.grpc.pub", msg.getTopic());
            assertEquals("from-java-grpc", new String(msg.getPayload(), StandardCharsets.UTF_8));
        }
    }

    @Test
    void grpcNodeSubscribeAndService() throws Exception {
        try (TestBus bus = TestBus.start();
                Publisher pub = new Publisher(bus.messageXsub);
                Node server = bus.makeNode("svc_server");
                Node client = Node.grpcAt("grpc_client", bus.grpcUrl())) {
            server.createService("svc.java_grpc_echo", body -> {
                byte[] prefix = "echo:".getBytes(StandardCharsets.UTF_8);
                byte[] out = new byte[prefix.length + body.length];
                System.arraycopy(prefix, 0, out, 0, prefix.length);
                System.arraycopy(body, 0, out, prefix.length, body.length);
                return out;
            });
            server.start();
            Thread.sleep(200);

            AtomicReference<String> gotTopic = new AtomicReference<>();
            AtomicReference<byte[]> gotPayload = new AtomicReference<>();
            client.createSubscription(
                    "java.grpc.topic",
                    (topic, payload) -> {
                        gotTopic.set(topic);
                        gotPayload.set(payload);
                    });
            Thread.sleep(300);

            pub.publish("java.grpc.topic", "hello-java-grpc".getBytes(StandardCharsets.UTF_8));
            assertTrue(
                    waitUntil(
                            () -> {
                                client.spinOnce(0.05);
                                return gotPayload.get() != null;
                            },
                            5000));
            assertEquals("java.grpc.topic", gotTopic.get());
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
                Node client = Node.grpcAt("grpc_action", bus.grpcUrl())) {
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
            List<ActionMessage> messages =
                    action.sendGoal("fly".getBytes(StandardCharsets.UTF_8), null, 5.0);
            assertEquals(3, messages.size());
            assertEquals("FEEDBACK", messages.get(0).getKind());
            assertEquals("step-1", new String(messages.get(0).getBody(), StandardCharsets.UTF_8));
            assertEquals("RESULT", messages.get(2).getKind());
            assertEquals("done:fly", new String(messages.get(2).getBody(), StandardCharsets.UTF_8));

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
