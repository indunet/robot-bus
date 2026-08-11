package org.indunet.robot.bus

/** HTTP discovery options (maps to C `RobotBusDiscoverOpts`: GET /api/v1/discover). */
class DiscoverOpts
@JvmOverloads
constructor(
    private val apiUrl: String? = null,
    private val brokerId: String? = null,
    private val timeoutSecs: Double = 0.0,
) {
    fun getApiUrl(): String? = apiUrl

    fun getBrokerId(): String? = brokerId

    fun getTimeoutSecs(): Double = timeoutSecs

    internal fun toNative(): RobotBusC.DiscoverOpts {
        val o = RobotBusC.DiscoverOpts()
        o.apiUrl = apiUrl
        o.brokerId = brokerId
        o.timeoutSecs = timeoutSecs
        o.write()
        return o
    }
}
