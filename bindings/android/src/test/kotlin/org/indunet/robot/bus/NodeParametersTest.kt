package org.indunet.robot.bus

import java.nio.charset.StandardCharsets
import java.nio.file.Files
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NodeParametersTest {
    @Test
    fun declareGetSetListAndYaml() {
        Node("params").use { node ->
            node.declareParameter("max_speed", 1.5)
            node.declareParameter("frame_id", "base_link")
            node.declareParameter("enabled", true)
            node.declareParameter("count", 3)

            assertEquals(1.5, node.getParameterValue("max_speed") as Double, 1e-9)
            assertEquals("base_link", node.getParameterValue("frame_id"))
            assertEquals(true, node.getParameterValue("enabled"))
            assertEquals(3L, node.getParameterValue("count"))
            assertEquals("max_speed", node.getParameter("max_speed").name)
            assertTrue(node.hasParameter("frame_id"))
            assertFalse(node.hasParameter("missing"))

            node.setParameter("max_speed", 2.0)
            assertEquals(2.0, node.getParameterValue("max_speed") as Double, 1e-9)

            val listed = node.listParameters()
            assertEquals(4, listed.names.size)
            assertEquals(4, node.listAllParameters().size)

            node.undeclareParameter("enabled")
            assertFalse(node.hasParameter("enabled"))
            assertEquals(3, node.listAllParameters().size)

            node.loadParametersFromYamlStr("ros__parameters:\n  max_speed: 3.25\n  extra: hello\n")
            assertEquals(3.25, node.getParameterValue("max_speed") as Double, 1e-9)
            assertEquals("hello", node.getParameterValue("extra"))

            val path = Files.createTempFile("robot_bus_params", ".yaml")
            try {
                Files.write(path, "count: 9\n".toByteArray(StandardCharsets.UTF_8))
                node.loadParametersFromYaml(path.toString())
                assertEquals(9L, node.getParameterValue("count"))
            } finally {
                Files.deleteIfExists(path)
            }
        }
    }
}
