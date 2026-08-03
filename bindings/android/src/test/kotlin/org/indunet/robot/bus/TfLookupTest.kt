package org.indunet.robot.bus

import org.indunet.robot.bus.geometry_msgs.msg.v1.Quaternion
import org.indunet.robot.bus.geometry_msgs.msg.v1.Transform
import org.indunet.robot.bus.geometry_msgs.msg.v1.TransformStamped
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3
import org.indunet.robot.bus.std_msgs.msg.v1.Header
import org.indunet.robot.bus.tf2_msgs.msg.v1.TFMessage
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TfLookupTest {
    @Test
    fun offlineBufferLookup() {
        TfBuffer().use { buf ->
            buf.setTransformMsg(staticEdge("base_link", "camera", 1.0, 0.0), true)
            assertTrue(buf.canTransform("base_link", "camera"))
            val t = buf.lookupTransform("base_link", "camera")
            assertEquals("camera", t.childFrameId)
            assertEquals(1.0, t.transform.translation.x, 1e-9)
        }
    }

    @Test
    fun listenerAgainstBroker() {
        TestBus.start().use { bus ->
            bus.makeNode("android-tf").use { node ->
                TfListener(node).use { listener ->
                    listener.buffer().use { buf ->
                        TransformBroadcaster(
                            node.createPublisher("/tf_static", TFMessage::class.java),
                        ).use { br ->
                            node.start()
                            Thread.sleep(200)

                            br.send(staticEdge("odom", "base_link", 0.0, 2.0))
                            assertTrue(waitUntil({ buf.canTransform("odom", "base_link") }, 3000))
                            val t = buf.lookupTransform("odom", "base_link")
                            assertEquals(2.0, t.transform.translation.y, 1e-9)

                            node.shutdown()
                            node.waitForShutdown()
                        }
                    }
                }
            }
        }
    }

    private fun staticEdge(parent: String, child: String, x: Double, y: Double): TFMessage =
        TFMessage.newBuilder()
            .addTransforms(
                TransformStamped.newBuilder()
                    .setHeader(Header.newBuilder().setFrameId(parent))
                    .setChildFrameId(child)
                    .setTransform(
                        Transform.newBuilder()
                            .setTranslation(Vector3.newBuilder().setX(x).setY(y).build())
                            .setRotation(Quaternion.newBuilder().setW(1.0).build())
                            .build(),
                    )
                    .build(),
            )
            .build()

    private fun waitUntil(pred: () -> Boolean, timeoutMs: Long): Boolean {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            if (pred()) return true
            Thread.sleep(20)
        }
        return pred()
    }
}
