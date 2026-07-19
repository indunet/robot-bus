package org.indunet.robot.bus

import com.sun.jna.Pointer
import com.sun.jna.ptr.LongByReference
import com.sun.jna.ptr.PointerByReference

/** Standalone sub socket (not tied to a [Node]). */
class Subscriber
@JvmOverloads
constructor(endpoint: String? = null) : AutoCloseable {
    private var ptr: Pointer? =
        Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_subscriber_new(endpoint), "Subscriber")

    fun subscribe(topic: String) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_subscriber_subscribe(ptr, topic),
            "subscribe",
        )
    }

    fun unsubscribe(topic: String) {
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_subscriber_unsubscribe(ptr, topic),
            "unsubscribe",
        )
    }

    /**
     * Receive one message.
     *
     * @param timeoutSecs negative blocks until a message arrives; use [receive] for that default
     */
    @JvmOverloads
    fun receive(timeoutSecs: Double = -1.0): TopicMessage {
        val outTopic = PointerByReference()
        val outData = PointerByReference()
        val outLen = LongByReference()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_subscriber_receive(
                ptr,
                timeoutSecs,
                outTopic,
                outData,
                outLen,
            ),
            "receive",
        )
        val topic = NativeUtils.takeCString(outTopic.value)
        val payload = NativeUtils.readBytes(outData.value, outLen.value)
        return TopicMessage(topic, payload)
    }

    fun endpoint(): String =
        NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_subscriber_endpoint(ptr))

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_subscriber_free(ptr)
        ptr = null
    }
}
