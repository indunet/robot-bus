package org.indunet.robot.bus

import com.sun.jna.Pointer
import com.sun.jna.ptr.LongByReference
import com.sun.jna.ptr.PointerByReference

/** Client for a named service on a [Node]. */
class ServiceClient internal constructor(private var ptr: Pointer?) : AutoCloseable {
    fun serviceName(): String =
        NativeUtils.takeCString(
            RobotBusC.Holder.INSTANCE.robot_bus_service_client_service_name(ptr),
        )

    @JvmOverloads
    fun call(body: ByteArray, timeoutSecs: Double = -1.0): ByteArray {
        val outData = PointerByReference()
        val outLen = LongByReference()
        Errors.check(
            RobotBusC.Holder.INSTANCE.robot_bus_service_client_call(
                ptr,
                body,
                body.size.toLong(),
                timeoutSecs,
                outData,
                outLen,
            ),
            "service call",
        )
        return NativeUtils.readBytes(outData.value, outLen.value)
    }

    override fun close() {
        RobotBusC.Holder.INSTANCE.robot_bus_service_client_free(ptr)
        ptr = null
    }
}
