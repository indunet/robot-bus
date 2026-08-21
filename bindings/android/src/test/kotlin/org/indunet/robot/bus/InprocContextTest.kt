package org.indunet.robot.bus

import java.io.IOException
import java.net.ServerSocket
import java.nio.charset.StandardCharsets
import java.util.concurrent.atomic.AtomicInteger
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Same-process inproc requires a shared [Context] with the embedded broker. */
class InprocContextTest {
    @Test
    fun inprocPubSubWithSharedContext() {
        Context().use { ctx ->
            Broker(ctx, inprocBrokerOptions()).use { _ ->
                Node.inproc(ctx, "inproc-sub").use { sub ->
                    Node.inproc(ctx, "inproc-pub").use { pub ->
                        Thread.sleep(150)

                        val hits = AtomicInteger()
                        sub.createSubscription(
                            "/inproc/demo",
                            { _, payload ->
                                assertEquals(
                                    "hello-inproc",
                                    String(payload, StandardCharsets.UTF_8),
                                )
                                hits.incrementAndGet()
                            },
                        )
                        sub.start()
                        Thread.sleep(100)

                        pub.createPublisher("/inproc/demo").use { topic ->
                            val deadline = System.currentTimeMillis() + 5000
                            while (hits.get() < 1 && System.currentTimeMillis() < deadline) {
                                topic.publish("hello-inproc".toByteArray(StandardCharsets.UTF_8))
                                Thread.sleep(20)
                            }
                        }

                        assertTrue("expected at least one inproc message", hits.get() >= 1)
                        sub.shutdown()
                        sub.waitForShutdown()
                    }
                }
            }
        }
    }

    @Test
    fun inprocActionGoalHandle() {
        Context().use { ctx ->
            Broker(ctx, inprocBrokerOptions()).use { _ ->
                Node.inproc(ctx, "inproc-action-server").use { server ->
                    Node.inproc(ctx, "inproc-action-client").use { clientNode ->
                        Thread.sleep(150)

                        server.createActionServer(
                            "/inproc/action",
                            ActionHandler { body ->
                                val payload = String(body, StandardCharsets.UTF_8)
                                listOf(
                                    ActionPhase(
                                        "FEEDBACK",
                                        "step:$payload".toByteArray(StandardCharsets.UTF_8),
                                    ),
                                    ActionPhase(
                                        "RESULT",
                                        "done:$payload".toByteArray(StandardCharsets.UTF_8),
                                    ),
                                )
                            },
                        )
                        server.start()
                        Thread.sleep(100)

                        val feedback = mutableListOf<String>()
                        clientNode.createActionClient("/inproc/action").use { action ->
                            action.sendGoal(
                                "move".toByteArray(StandardCharsets.UTF_8),
                                null,
                                3.0,
                            ) { message ->
                                feedback.add(String(message.body, StandardCharsets.UTF_8))
                            }.use { goal ->
                                assertEquals("/inproc/action", goal.actionName())
                                assertTrue(goal.goalId().isNotEmpty())
                                assertEquals(
                                    "done:move",
                                    String(goal.result(3.0).body, StandardCharsets.UTF_8),
                                )
                                assertEquals(listOf("step:move"), feedback)
                            }
                        }

                        server.shutdown()
                        server.waitForShutdown()
                    }
                }
            }
        }
    }

    /** Ephemeral TCP binds, but keep inproc (tcpOnly=false). */
    @Throws(IOException::class)
    private fun inprocBrokerOptions(): BrokerOptions =
        BrokerOptions(
            messageXsubBind = "tcp://127.0.0.1:${freePort()}",
            messageXpubBind = "tcp://127.0.0.1:${freePort()}",
            serviceFrontendBind = "tcp://127.0.0.1:${freePort()}",
            serviceBackendBind = "tcp://127.0.0.1:${freePort()}",
            actionFrontendBind = "tcp://127.0.0.1:${freePort()}",
            actionBackendBind = "tcp://127.0.0.1:${freePort()}",
            apiListen = "127.0.0.1:${freePort()}",
            consoleListen = null,
            tcpOnly = false,
            noConsole = true,
        )

    @Throws(IOException::class)
    private fun freePort(): Int =
        ServerSocket(0).use { socket ->
            socket.reuseAddress = true
            socket.localPort
        }
}
