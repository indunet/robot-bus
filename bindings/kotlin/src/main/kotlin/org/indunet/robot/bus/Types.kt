package org.indunet.robot.bus

import com.sun.jna.Pointer
import com.sun.jna.ptr.LongByReference
import com.sun.jna.ptr.PointerByReference

data class TopicMessage(val topic: String, val payload: ByteArray) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is TopicMessage) return false
        return topic == other.topic && payload.contentEquals(other.payload)
    }

    override fun hashCode(): Int = 31 * topic.hashCode() + payload.contentHashCode()
}

data class ActionMessage(
    val kind: String,
    val body: ByteArray,
    val goalId: String,
    val actionName: String,
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is ActionMessage) return false
        return kind == other.kind &&
            body.contentEquals(other.body) &&
            goalId == other.goalId &&
            actionName == other.actionName
    }

    override fun hashCode(): Int {
        var result = kind.hashCode()
        result = 31 * result + body.contentHashCode()
        result = 31 * result + goalId.hashCode()
        result = 31 * result + actionName.hashCode()
        return result
    }
}

data class ActionPhase(val phase: String, val body: ByteArray) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is ActionPhase) return false
        return phase == other.phase && body.contentEquals(other.body)
    }

    override fun hashCode(): Int = 31 * phase.hashCode() + body.contentHashCode()
}

enum class CallbackGroupType(val code: Int) {
    MutuallyExclusive(0),
    Reentrant(1),
}

fun messageXsubEndpoint(host: String = "localhost", transport: String = "tcp"): String =
    endpointCall { out -> RobotBusC.INSTANCE.robot_bus_message_xsub_endpoint(host, transport, out) }

fun messageXpubEndpoint(host: String = "localhost", transport: String = "tcp"): String =
    endpointCall { out -> RobotBusC.INSTANCE.robot_bus_message_xpub_endpoint(host, transport, out) }

class Publisher(endpoint: String? = null) : AutoCloseable {
    private var ptr: Pointer? =
        checkPtr(RobotBusC.INSTANCE.robot_bus_publisher_new(endpoint), "Publisher")

    fun publish(topic: String, payload: ByteArray) {
        check(
            RobotBusC.INSTANCE.robot_bus_publisher_publish(ptr, topic, payload, payload.size.toLong()),
            "publish",
        )
    }

    fun endpoint(): String = takeCString(RobotBusC.INSTANCE.robot_bus_publisher_endpoint(ptr))

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_publisher_free(ptr)
        ptr = null
    }
}

class Subscriber(endpoint: String? = null) : AutoCloseable {
    private var ptr: Pointer? =
        checkPtr(RobotBusC.INSTANCE.robot_bus_subscriber_new(endpoint), "Subscriber")

    fun subscribe(topic: String) {
        check(RobotBusC.INSTANCE.robot_bus_subscriber_subscribe(ptr, topic), "subscribe")
    }

    fun unsubscribe(topic: String) {
        check(RobotBusC.INSTANCE.robot_bus_subscriber_unsubscribe(ptr, topic), "unsubscribe")
    }

    /** `timeoutSecs` null / negative blocks until a message arrives. */
    fun receive(timeoutSecs: Double? = null): TopicMessage {
        val outTopic = PointerByReference()
        val outData = PointerByReference()
        val outLen = LongByReference()
        val timeout = timeoutSecs ?: -1.0
        check(
            RobotBusC.INSTANCE.robot_bus_subscriber_receive(ptr, timeout, outTopic, outData, outLen),
            "receive",
        )
        val topic = takeCString(outTopic.value)
        val payload = readBytes(outData.value, outLen.value)
        return TopicMessage(topic, payload)
    }

    fun endpoint(): String = takeCString(RobotBusC.INSTANCE.robot_bus_subscriber_endpoint(ptr))

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_subscriber_free(ptr)
        ptr = null
    }
}

class TopicPublisher internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun topic(): String = takeCString(RobotBusC.INSTANCE.robot_bus_topic_publisher_topic(ptr))

    fun publish(payload: ByteArray) {
        check(
            RobotBusC.INSTANCE.robot_bus_topic_publisher_publish(ptr, payload, payload.size.toLong()),
            "publish",
        )
    }

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_topic_publisher_free(ptr)
        ptr = null
    }
}

class ServiceClient internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun serviceName(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_service_client_service_name(ptr))

    fun call(body: ByteArray, timeoutSecs: Double = -1.0): ByteArray {
        val outData = PointerByReference()
        val outLen = LongByReference()
        check(
            RobotBusC.INSTANCE.robot_bus_service_client_call(
                ptr,
                body,
                body.size.toLong(),
                timeoutSecs,
                outData,
                outLen,
            ),
            "service call",
        )
        return readBytes(outData.value, outLen.value)
    }

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_service_client_free(ptr)
        ptr = null
    }
}

class ActionClient internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun actionName(): String =
        takeCString(RobotBusC.INSTANCE.robot_bus_action_client_action_name(ptr))

    fun sendGoal(
        body: ByteArray,
        goalId: String? = null,
        timeoutSecs: Double = -1.0,
    ): List<ActionMessage> {
        val outMsgs = PointerByReference()
        val outCount = LongByReference()
        check(
            RobotBusC.INSTANCE.robot_bus_action_client_send_goal(
                ptr,
                body,
                body.size.toLong(),
                goalId,
                timeoutSecs,
                outMsgs,
                outCount,
            ),
            "send_goal",
        )
        val count = outCount.value
        val base = outMsgs.value
        if (base == null || count <= 0) return emptyList()
        return try {
            val size = RobotBusC.ActionMessageStruct().size().toLong()
            (0 until count).map { i ->
                val msg = RobotBusC.ActionMessageStruct(base.share(i * size))
                ActionMessage(
                    kind = msg.kind?.getString(0).orEmpty(),
                    body = if (msg.body != null && msg.bodyLen > 0) {
                        msg.body!!.getByteArray(0, msg.bodyLen.toInt())
                    } else {
                        ByteArray(0)
                    },
                    goalId = msg.goalId?.getString(0).orEmpty(),
                    actionName = msg.actionName?.getString(0).orEmpty(),
                )
            }
        } finally {
            RobotBusC.INSTANCE.robot_bus_action_messages_free(base, count)
        }
    }

    fun cancel(
        goalId: String,
        body: ByteArray = ByteArray(0),
        timeoutSecs: Double = -1.0,
    ): ActionMessage {
        val out = RobotBusC.ActionMessageStruct()
        check(
            RobotBusC.INSTANCE.robot_bus_action_client_cancel(
                ptr,
                goalId,
                body,
                body.size.toLong(),
                timeoutSecs,
                out,
            ),
            "cancel",
        )
        out.read()
        val result = ActionMessage(
            kind = out.kind?.getString(0).orEmpty(),
            body = if (out.body != null && out.bodyLen > 0) {
                out.body!!.getByteArray(0, out.bodyLen.toInt())
            } else {
                ByteArray(0)
            },
            goalId = out.goalId?.getString(0).orEmpty(),
            actionName = out.actionName?.getString(0).orEmpty(),
        )
        RobotBusC.INSTANCE.robot_bus_free_string(out.kind)
        RobotBusC.INSTANCE.robot_bus_free_bytes(out.body, out.bodyLen)
        RobotBusC.INSTANCE.robot_bus_free_string(out.goalId)
        RobotBusC.INSTANCE.robot_bus_free_string(out.actionName)
        return result
    }

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_action_client_free(ptr)
        ptr = null
    }
}

class ShutdownHandle internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun shutdown() {
        RobotBusC.INSTANCE.robot_bus_shutdown_handle_shutdown(ptr)
    }

    fun isRunning(): Boolean =
        RobotBusC.INSTANCE.robot_bus_shutdown_handle_is_running(ptr) != 0

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_shutdown_handle_free(ptr)
        ptr = null
    }
}

class CallbackGroup internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun id(): Long = RobotBusC.INSTANCE.robot_bus_callback_group_id(ptr)

    fun kind(): CallbackGroupType =
        when (RobotBusC.INSTANCE.robot_bus_callback_group_kind(ptr)) {
            1 -> CallbackGroupType.Reentrant
            else -> CallbackGroupType.MutuallyExclusive
        }

    internal fun raw(): Pointer? = ptr

    override fun close() {
        RobotBusC.INSTANCE.robot_bus_callback_group_free(ptr)
        ptr = null
    }
}

class TimerHandle internal constructor(internal val ptr: Pointer?) : AutoCloseable {
    override fun close() {
        RobotBusC.INSTANCE.robot_bus_timer_handle_free(ptr)
    }
}
