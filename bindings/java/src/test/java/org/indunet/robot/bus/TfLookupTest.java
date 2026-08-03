package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.indunet.robot.bus.geometry_msgs.msg.v1.Quaternion;
import org.indunet.robot.bus.geometry_msgs.msg.v1.Transform;
import org.indunet.robot.bus.geometry_msgs.msg.v1.TransformStamped;
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3;
import org.indunet.robot.bus.std_msgs.msg.v1.Header;
import org.indunet.robot.bus.tf2_msgs.msg.v1.TFMessage;
import org.junit.jupiter.api.Test;

class TfLookupTest {
    @Test
    void offlineBufferLookup() {
        try (TfBuffer buf = new TfBuffer()) {
            buf.setTransformMsg(staticEdge("base_link", "camera", 1.0, 0.0), true);
            assertTrue(buf.canTransform("base_link", "camera"));
            TransformStamped t = buf.lookupTransform("base_link", "camera");
            assertEquals("camera", t.getChildFrameId());
            assertEquals(1.0, t.getTransform().getTranslation().getX(), 1e-9);
        }
    }

    @Test
    void listenerAgainstBroker() throws Exception {
        try (TestBus bus = TestBus.start();
                Node node = bus.makeNode("java-tf");
                TfListener listener = new TfListener(node);
                TfBuffer buf = listener.buffer()) {
            TransformBroadcaster br =
                    new TransformBroadcaster(node.createPublisher("/tf_static", TFMessage.class));
            node.start();
            Thread.sleep(200);

            br.send(staticEdge("odom", "base_link", 0.0, 2.0));
            assertTrue(waitUntil(() -> buf.canTransform("odom", "base_link"), 3000));
            TransformStamped t = buf.lookupTransform("odom", "base_link");
            assertEquals(2.0, t.getTransform().getTranslation().getY(), 1e-9);

            node.shutdown();
            node.waitForShutdown();
            br.close();
        }
    }

    private static TFMessage staticEdge(String parent, String child, double x, double y) {
        return TFMessage.newBuilder()
                .addTransforms(
                        TransformStamped.newBuilder()
                                .setHeader(Header.newBuilder().setFrameId(parent))
                                .setChildFrameId(child)
                                .setTransform(
                                        Transform.newBuilder()
                                                .setTranslation(
                                                        Vector3.newBuilder().setX(x).setY(y).build())
                                                .setRotation(Quaternion.newBuilder().setW(1.0).build())
                                                .build())
                                .build())
                .build();
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
