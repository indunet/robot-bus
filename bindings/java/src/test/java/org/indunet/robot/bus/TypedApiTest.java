package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3;
import org.indunet.robot.bus.robot_bus_interface.action.v1.FibonacciFeedback;
import org.indunet.robot.bus.robot_bus_interface.action.v1.FibonacciGoal;
import org.indunet.robot.bus.robot_bus_interface.action.v1.FibonacciResult;
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolRequest;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolResponse;
import org.junit.jupiter.api.Test;

/**
 * Typed API coverage aligned with Python {@code test_typed_api.py} / C++ pub-sub & service
 * tests: codec unit checks + ephemeral in-process broker round-trips.
 */
class TypedApiTest {
    @Test
    void protoCodecRoundtripAndBadPayload() {
        Imu imu =
                Imu.newBuilder()
                        .setAngularVelocity(Vector3.newBuilder().setZ(0.1).build())
                        .build();
        byte[] bytes = ProtoCodec.encode(imu);
        Imu parsed = ProtoCodec.parse(Imu.class, bytes);
        assertEquals(0.1, parsed.getAngularVelocity().getZ(), 1e-9);

        assertNull(ProtoCodec.tryParse(Imu.class, new byte[] {0x01, 0x02, 0x03}));
        assertThrows(
                IllegalArgumentException.class,
                () -> ProtoCodec.requireMessageType(String.class, "msgType"));
    }

    @Test
    void typedPubSubAgainstBroker() throws Exception {
        try (TestBus bus = TestBus.start();
                Node node = bus.makeNode("typed-pubsub")) {
            AtomicReference<Imu> got = new AtomicReference<>();
            TypedTopicPublisher<Imu> pub = node.createPublisher("/imu", Imu.class);
            node.createSubscription("/imu", (topic, msg) -> got.set(msg), Imu.class);
            node.start();
            Thread.sleep(200);

            pub.publish(
                    Imu.newBuilder()
                            .setAngularVelocity(Vector3.newBuilder().setZ(0.25).build())
                            .build());

            // start() drives callbacks on a background thread — do not call spinOnce.
            assertTrue(waitUntil(() -> got.get() != null, 3000));
            assertEquals(0.25, got.get().getAngularVelocity().getZ(), 1e-9);
            node.shutdown();
            node.waitForShutdown();
        }
    }

    @Test
    void typedServiceAgainstBroker() throws Exception {
        try (TestBus bus = TestBus.start();
                Node server = bus.makeNode("typed-svc-server");
                Node clientNode = bus.makeNode("typed-svc-client")) {
            server.createService(
                    "/set_bool",
                    req ->
                            SetBoolResponse.newBuilder()
                                    .setSuccess(true)
                                    .setMessage("set:" + req.getData())
                                    .build(),
                    SetBoolRequest.class,
                    SetBoolResponse.class);
            server.start();
            Thread.sleep(100);

            TypedServiceClient<SetBoolRequest, SetBoolResponse> client =
                    clientNode.createClient("/set_bool", SetBoolRequest.class, SetBoolResponse.class);
            SetBoolResponse resp =
                    client.call(SetBoolRequest.newBuilder().setData(true).build(), 2.0);
            assertTrue(resp.getSuccess());
            assertEquals("set:true", resp.getMessage());

            server.shutdown();
            server.waitForShutdown();
        }
    }

    @Test
    void typedActionAgainstBroker() throws Exception {
        try (TestBus bus = TestBus.start();
                Node server = bus.makeNode("typed-act-server");
                Node clientNode = bus.makeNode("typed-act-client")) {
            server.createActionServer(
                    "/fibonacci",
                    goal ->
                            List.of(
                                    new TypedActionPhase(
                                            "FEEDBACK",
                                            FibonacciFeedback.newBuilder().addSequence(0).build()),
                                    new TypedActionPhase(
                                            "RESULT",
                                            FibonacciResult.newBuilder()
                                                    .addSequence(goal.getOrder())
                                                    .build())),
                    FibonacciGoal.class,
                    FibonacciFeedback.class,
                    FibonacciResult.class);
            server.start();
            Thread.sleep(100);

            TypedActionClient<FibonacciGoal, FibonacciFeedback, FibonacciResult> client =
                    clientNode.createActionClient(
                            "/fibonacci",
                            FibonacciGoal.class,
                            FibonacciFeedback.class,
                            FibonacciResult.class);
            List<FibonacciFeedback> feedback = new java.util.ArrayList<>();
            try (TypedActionGoalHandle<FibonacciFeedback, FibonacciResult> goal =
                    client.sendGoal(
                            FibonacciGoal.newBuilder().setOrder(5).build(),
                            null,
                            5.0,
                            feedback::add)) {
                FibonacciResult result = goal.result(5.0);
                assertEquals(1, feedback.size());
                assertEquals(1, result.getSequenceCount());
                assertEquals(5, result.getSequence(0));
            }

            server.shutdown();
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
