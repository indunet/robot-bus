package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Publisher bound to a topic on a [Node]. */
class TopicPublisher internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun topic(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_topic_publisher_topic(ptr))

    fun publish(payload: ByteArray) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_topic_publisher_publish(
                ptr,
                payload,
                payload.size.toLong(),
            ),
            "publish",
        )
    }

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_topic_publisher_free(ptr)
        ptr = null
    }
}
