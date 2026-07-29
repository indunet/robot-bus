package org.indunet.robot.bus

/** UDP multicast discovery options (maps to C `RobotBusDiscoverOpts`). */
class DiscoverOpts
@JvmOverloads
constructor(
    private val domainId: Int = 0,
    private val brokerId: String? = null,
    private val multicastAddr: String? = null,
    private val multicastPort: Int = 0,
    private val timeoutSecs: Double = 0.0,
) {
    fun getDomainId(): Int = domainId

    fun getBrokerId(): String? = brokerId

    fun getMulticastAddr(): String? = multicastAddr

    fun getMulticastPort(): Int = multicastPort

    fun getTimeoutSecs(): Double = timeoutSecs

    internal fun toNative(): RobotBusC.DiscoverOpts {
        val o = RobotBusC.DiscoverOpts()
        o.domainId = domainId
        o.brokerId = brokerId
        o.multicastAddr = multicastAddr
        o.multicastPort = (multicastPort and 0xffff).toShort()
        o.timeoutSecs = timeoutSecs
        o.write()
        return o
    }
}
