package org.indunet.robot.bus

import com.google.protobuf.MessageLite

/** Publisher that accepts protobuf message instances. */
class TypedTopicPublisher<T : MessageLite> internal constructor(
    private val inner: TopicPublisher,
    private val msgType: Class<T>,
) : AutoCloseable {
    fun topic(): String = inner.topic()

    fun msgType(): Class<T> = msgType

    fun publish(msg: T) {
        requireNotNull(msg) { "msg" }
        if (!msgType.isInstance(msg)) {
            throw IllegalArgumentException(
                "publisher for ${msgType.simpleName} got ${msg.javaClass.simpleName}",
            )
        }
        inner.publish(ProtoCodec.encode(msg))
    }

    override fun close() {
        inner.close()
    }

    override fun toString(): String =
        "TypedTopicPublisher{topic=${topic()}, msgType=${msgType.simpleName}}"
}
