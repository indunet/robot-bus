package org.indunet.robot.bus

import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference

internal fun takeCString(p: Pointer?): String {
    if (p == null) return ""
    return try {
        p.getString(0)
    } finally {
        RobotBusC.INSTANCE.robot_bus_free_string(p)
    }
}

internal fun readBytes(p: Pointer?, len: Long): ByteArray {
    if (p == null || len <= 0) return ByteArray(0)
    return try {
        p.getByteArray(0, len.toInt())
    } finally {
        RobotBusC.INSTANCE.robot_bus_free_bytes(p, len)
    }
}

internal fun allocReplyBytes(payload: ByteArray): Pointer? {
    if (payload.isEmpty()) return null
    val buf = RobotBusC.INSTANCE.robot_bus_alloc_bytes(payload.size.toLong())
        ?: throw RobotBusException("robot_bus_alloc_bytes failed")
    buf.write(0, payload, 0, payload.size)
    return buf
}

internal fun endpointCall(block: (PointerByReference) -> Int): String {
    val out = PointerByReference()
    check(block(out), "endpoint")
    return takeCString(out.value)
}
