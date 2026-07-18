package org.indunet.robot.bus

class RobotBusException(message: String) : RuntimeException(message)

internal fun lastError(): String = RobotBusC.INSTANCE.robot_bus_last_error().orEmpty()

internal fun check(rc: Int, what: String) {
    if (rc < 0) {
        val err = lastError()
        throw RobotBusException("$what: ${err.ifEmpty { "unknown error" }}")
    }
}

internal fun checkPtr(p: com.sun.jna.Pointer?, what: String): com.sun.jna.Pointer {
    if (p == null) {
        val err = lastError()
        throw RobotBusException("$what: ${err.ifEmpty { "null" }}")
    }
    return p
}
