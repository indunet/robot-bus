package org.indunet.robot.bus

import org.indunet.robot.bus.geometry_msgs.msg.v1.TransformStamped
import org.indunet.robot.bus.tf2_msgs.msg.v1.TFMessage

/** Thin helper over a typed `TFMessage` publisher. */
class TransformBroadcaster(
    private val publisher: TypedTopicPublisher<TFMessage>,
) : AutoCloseable {
    fun send(msg: TFMessage) {
        publisher.publish(msg)
    }

    fun send(vararg transforms: TransformStamped) {
        val b = TFMessage.newBuilder()
        for (t in transforms) {
            b.addTransforms(t)
        }
        publisher.publish(b.build())
    }

    override fun close() {
        publisher.close()
    }
}
