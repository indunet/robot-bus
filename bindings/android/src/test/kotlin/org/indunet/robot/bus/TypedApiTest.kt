package org.indunet.robot.bus

import java.util.concurrent.atomic.AtomicReference
import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Typed pub-sub smoke against an ephemeral in-process broker (desktop native). */
class TypedApiTest {
    @Test
    fun actionGoalHandleStreamsFeedback() {
        TestBus.start().use { bus ->
            bus.makeNode("action-server").use { server ->
                bus.makeNode("action-client").use { clientNode ->
                    server.createActionServer(
                        "/demo",
                        ActionHandler { body ->
                            listOf(
                                ActionPhase("FEEDBACK", "step".toByteArray()),
                                ActionPhase("RESULT", "done:${body.decodeToString()}".toByteArray()),
                            )
                        },
                    )
                    server.start()
                    Thread.sleep(200)

                    val feedback = AtomicReference<String?>()
                    clientNode.createActionClient("/demo").use { client ->
                        client.sendGoal("fly".toByteArray()) { message ->
                            feedback.set(message.body.decodeToString())
                        }.use { goal ->
                            assertTrue(goal.goalId().isNotEmpty())
                            assertEquals("done:fly", goal.result(3.0).body.decodeToString())
                            assertEquals("step", feedback.get())
                        }
                    }
                    server.shutdown()
                    server.waitForShutdown()
                }
            }
        }
    }

    @Test
    fun typedPubSubAgainstBroker() {
        TestBus.start().use { bus ->
            bus.makeNode("typed-pubsub").use { node ->
                val got = AtomicReference<Imu?>()
                val pub = node.createPublisher("/imu", Imu::class.java)
                node.createSubscription(
                    "/imu",
                    { msg -> got.set(msg) },
                    Imu::class.java,
                )
                node.start()
                Thread.sleep(200)

                pub.publish(
                    Imu.newBuilder()
                        .setAngularVelocity(Vector3.newBuilder().setZ(0.25).build())
                        .build(),
                )

                assertTrue(waitUntil({ got.get() != null }, 3000))
                assertEquals(0.25, got.get()!!.angularVelocity.z, 1e-9)
                node.shutdown()
                node.waitForShutdown()
            }
        }
    }

    private fun waitUntil(pred: () -> Boolean, timeoutMs: Long): Boolean {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            if (pred()) return true
            Thread.sleep(20)
        }
        return pred()
    }
}
