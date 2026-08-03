package org.indunet.robot.bus

import com.sun.jna.Pointer
import com.sun.jna.ptr.LongByReference
import com.sun.jna.ptr.PointerByReference
import org.indunet.robot.bus.geometry_msgs.msg.v1.TransformStamped
import org.indunet.robot.bus.tf2_msgs.msg.v1.TFMessage

/** In-memory TF tree buffer (static + latest dynamic edges). */
class TfBuffer : AutoCloseable {
    private var ptr: Pointer?

    constructor() {
        ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_new(), "TfBuffer")
    }

    internal constructor(ptr: Pointer) {
        this.ptr = ptr
    }

    fun clear() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_clear(ptr), "TfBuffer.clear")
    }

    /** Ingest a `tf2_msgs/TFMessage`. `isStatic` marks `/tf_static` traffic. */
    fun setTransformMsg(msg: TFMessage, isStatic: Boolean) {
        val bytes = ProtoCodec.encode(msg)
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_set_transform_msg(
                ptr,
                bytes,
                bytes.size.toLong(),
                if (isStatic) 1 else 0,
            ),
            "TfBuffer.setTransformMsg",
        )
    }

    /** Lookup transform of `source` relative to `target`. */
    fun lookupTransform(target: String, source: String): TransformStamped {
        val outData = PointerByReference()
        val outLen = LongByReference()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_lookup_transform(
                ptr,
                target,
                source,
                outData,
                outLen,
            ),
            "TfBuffer.lookupTransform",
        )
        val bytes = NativeUtils.readBytes(outData.value, outLen.value)
        return ProtoCodec.parse(TransformStamped::class.java, bytes)
    }

    fun canTransform(target: String, source: String): Boolean =
        RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_can_transform(ptr, target, source) != 0

    fun frames(): List<String> {
        val joined = NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_frames(ptr))
        if (joined.isEmpty()) return emptyList()
        return joined.split('\n').filter { it.isNotEmpty() }
    }

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_free(ptr)
        ptr = null
    }
}
