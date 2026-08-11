package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;

class NodeParametersTest {
    @Test
    void declareGetSetListAndYaml() throws Exception {
        try (Node node = new Node("params")) {
            node.declareParameter("max_speed", 1.5);
            node.declareParameter("frame_id", "base_link");
            node.declareParameter("enabled", true);
            node.declareParameter("count", 3);

            assertEquals(1.5, (Double) node.getParameterValue("max_speed"), 1e-9);
            assertEquals("base_link", node.getParameterValue("frame_id"));
            assertEquals(true, node.getParameterValue("enabled"));
            assertEquals(3L, node.getParameterValue("count"));
            assertEquals("max_speed", node.getParameter("max_speed").getName());
            assertTrue(node.hasParameter("frame_id"));
            assertFalse(node.hasParameter("missing"));

            node.setParameter("max_speed", 2.0);
            assertEquals(2.0, (Double) node.getParameterValue("max_speed"), 1e-9);

            ListParametersResult listed = node.listParameters();
            assertEquals(4, listed.getNames().size());
            assertEquals(4, node.listAllParameters().size());

            node.undeclareParameter("enabled");
            assertFalse(node.hasParameter("enabled"));
            assertEquals(3, node.listAllParameters().size());

            node.loadParametersFromYamlStr("ros__parameters:\n  max_speed: 3.25\n  extra: hello\n");
            assertEquals(3.25, (Double) node.getParameterValue("max_speed"), 1e-9);
            assertEquals("hello", node.getParameterValue("extra"));

            Path path = Files.createTempFile("robot_bus_params", ".yaml");
            try {
                Files.write(path, "count: 9\n".getBytes(StandardCharsets.UTF_8));
                node.loadParametersFromYaml(path.toString());
                assertEquals(9L, node.getParameterValue("count"));
            } finally {
                Files.deleteIfExists(path);
            }
        }
    }
}
