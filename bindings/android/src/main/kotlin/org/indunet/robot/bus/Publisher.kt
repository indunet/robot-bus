package org.indunet.robot.bus

import com.sun.jna.Pointer

/** Standalone pub socket (not tied to a [Node]). */
class Publisher
@JvmOverloads
constructor(endpoint: String? = null) : AutoCloseable {
    private var ptr: Pointer? =
        Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_publisher_new(endpoint), "Publisher")

    fun publish(topic: String, payload: ByteArray) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_publisher_publish(
                ptr,
                topic,
                payload,
                payload.size.toLong(),
            ),
            "publish",
        )
    }

    fun endpoint(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_publisher_endpoint(ptr))

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_publisher_free(ptr)
        ptr = null
    }
}
