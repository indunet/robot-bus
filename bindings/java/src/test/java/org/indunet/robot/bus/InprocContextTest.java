package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;

/** Same-process inproc requires a shared {@link Context} with the embedded broker. */
class InprocContextTest {
    @Test
    void inprocPubSubWithSharedContext() throws Exception {
        try (Context ctx = new Context();
                Broker broker = new Broker(ctx, inprocBrokerOptions());
                Node sub = Node.inproc(ctx, "inproc-sub");
                Node pub = Node.inproc(ctx, "inproc-pub")) {
            Thread.sleep(150);

            AtomicInteger hits = new AtomicInteger();
            sub.createSubscription(
                    "/inproc/demo",
                    (payload) -> {
                        assertEquals("hello-inproc", new String(payload, StandardCharsets.UTF_8));
                        hits.incrementAndGet();
                    });
            sub.start();
            Thread.sleep(100);

            try (TopicPublisher topic = pub.createPublisher("/inproc/demo")) {
                long deadline = System.currentTimeMillis() + 5000;
                while (hits.get() < 1 && System.currentTimeMillis() < deadline) {
                    topic.publish("hello-inproc".getBytes(StandardCharsets.UTF_8));
                    Thread.sleep(20);
                }
            }

            assertTrue(hits.get() >= 1, "expected at least one inproc message");
            sub.shutdown();
            sub.waitForShutdown();
        }
    }

    @Test
    void inprocActionGoalHandle() throws Exception {
        try (Context ctx = new Context();
                Broker broker = new Broker(ctx, inprocBrokerOptions());
                Node server = Node.inproc(ctx, "inproc-action-server");
                Node clientNode = Node.inproc(ctx, "inproc-action-client")) {
            Thread.sleep(150);

            server.createActionServer(
                    "/inproc/action",
                    body ->
                            List.of(
                                    new ActionPhase(
                                            "FEEDBACK",
                                            ("step:" + new String(body, StandardCharsets.UTF_8))
                                                    .getBytes(StandardCharsets.UTF_8)),
                                    new ActionPhase(
                                            "RESULT",
                                            ("done:" + new String(body, StandardCharsets.UTF_8))
                                                    .getBytes(StandardCharsets.UTF_8))));
            server.start();
            Thread.sleep(100);

            List<String> feedback = new ArrayList<>();
            try (ActionClient action = clientNode.createActionClient("/inproc/action");
                    ActionGoalHandle goal =
                            action.sendGoal(
                                    "move".getBytes(StandardCharsets.UTF_8),
                                    null,
                                    3.0,
                                    message ->
                                            feedback.add(
                                                    new String(
                                                            message.getBody(),
                                                            StandardCharsets.UTF_8)))) {
                assertEquals("/inproc/action", goal.actionName());
                assertFalse(goal.goalId().isEmpty());
                assertEquals(
                        "done:move",
                        new String(goal.result(3.0).getBody(), StandardCharsets.UTF_8));
                assertEquals(List.of("step:move"), feedback);
            }

            server.shutdown();
            server.waitForShutdown();
        }
    }

    /** Ephemeral TCP binds (`:0`), but keep inproc (tcpOnly=false). */
    private static BrokerOptions inprocBrokerOptions() {
        return new BrokerOptions(
                "tcp://127.0.0.1:0",
                "tcp://127.0.0.1:0",
                "tcp://127.0.0.1:0",
                "tcp://127.0.0.1:0",
                "tcp://127.0.0.1:0",
                "tcp://127.0.0.1:0",
                "127.0.0.1:0",
                null,
                false,
                true);
    }
}
