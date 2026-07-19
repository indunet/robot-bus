package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.Test;

class NodeParametersTest {
    @Test
    void declareGetSetListAndYaml() throws Exception {
        try (Node node = new Node("params")) {
            node.declareParameter("max_speed", 1.5);
            node.declareParameter("frame_id", "base_link");
            node.declareParameter("enabled", true);
            node.declareParameter("count", 3);

            assertEquals(1.5, (Double) node.getParameter("max_speed"), 1e-9);
            assertEquals("base_link", node.getParameter("frame_id"));
            assertEquals(true, node.getParameter("enabled"));
            assertEquals(3L, node.getParameter("count"));
            assertTrue(node.hasParameter("frame_id"));
            assertFalse(node.hasParameter("missing"));

            node.setParameter("max_speed", 2.0);
            assertEquals(2.0, (Double) node.getParameter("max_speed"), 1e-9);

            List<Parameter> listed = node.listParameters();
            assertEquals(4, listed.size());

            node.loadParametersFromYamlStr("ros__parameters:\n  max_speed: 3.25\n  extra: hello\n");
            assertEquals(3.25, (Double) node.getParameter("max_speed"), 1e-9);
            assertEquals("hello", node.getParameter("extra"));

            Path path = Files.createTempFile("robot_bus_params", ".yaml");
            try {
                Files.write(path, "count: 9\n".getBytes(StandardCharsets.UTF_8));
                node.loadParametersFromYaml(path.toString());
                assertEquals(9L, node.getParameter("count"));
            } finally {
                Files.deleteIfExists(path);
            }
        }
    }
}
