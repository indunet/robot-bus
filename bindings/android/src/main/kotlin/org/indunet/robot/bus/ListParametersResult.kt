package org.indunet.robot.bus

/** ROS-shaped result of [Node.listParameters]. */
data class ListParametersResult(
    val names: List<String>,
    val prefixes: List<String>,
)
