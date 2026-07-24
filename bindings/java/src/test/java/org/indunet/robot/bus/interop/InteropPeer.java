package org.indunet.robot.bus.interop;

import java.util.concurrent.atomic.AtomicReference;
import org.indunet.robot.bus.Node;
import org.indunet.robot.bus.NodeOptions;
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolRequest;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolResponse;

/**
 * Env-driven interop peer for {@code tests/interop/}.
 *
 * <p>Roles: {@code sub} (Imu), {@code svc-server} (SetBool).
 */
public final class InteropPeer {
    private static final String TOPIC = "/interop/imu";
    private static final String SERVICE = "/interop/set_bool";
    private static final double EXPECT_Z = 0.42;

    private InteropPeer() {}

    public static void main(String[] args) throws Exception {
        String role = requireEnv("ROBOT_BUS_INTEROP_ROLE");
        NodeOptions opts = optionsFromEnv();
        switch (role) {
            case "sub":
                runSub(opts);
                break;
            case "svc-server":
                runSvcServer(opts);
                break;
            default:
                throw new IllegalArgumentException("unknown ROBOT_BUS_INTEROP_ROLE: " + role);
        }
    }

    private static void runSub(NodeOptions opts) throws Exception {
        AtomicReference<Imu> got = new AtomicReference<>();
        try (Node node = new Node("interop_java_sub", opts)) {
            node.createSubscription(TOPIC, (topic, msg) -> got.set(msg), Imu.class);
            node.start();
            System.out.println("READY");
            long deadline = System.currentTimeMillis() + 8000;
            while (got.get() == null && System.currentTimeMillis() < deadline) {
                Thread.sleep(20);
            }
            Imu imu = got.get();
            if (imu == null) {
                throw new IllegalStateException("timed out waiting for Imu on " + TOPIC);
            }
            double z = imu.getAngularVelocity().getZ();
            if (Math.abs(z - EXPECT_Z) > 1e-9) {
                throw new IllegalStateException("unexpected z=" + z);
            }
            node.shutdown();
            node.waitForShutdown();
        }
    }

    private static void runSvcServer(NodeOptions opts) throws Exception {
        try (Node node = new Node("interop_java_svc", opts)) {
            node.createService(
                    SERVICE,
                    req ->
                            SetBoolResponse.newBuilder()
                                    .setSuccess(true)
                                    .setMessage("set:" + (req.getData() ? "true" : "false"))
                                    .build(),
                    SetBoolRequest.class,
                    SetBoolResponse.class);
            node.start();
            System.out.println("READY");
            Thread.sleep(15_000);
            node.shutdown();
            node.waitForShutdown();
        }
    }

    private static NodeOptions optionsFromEnv() {
        return new NodeOptions(
                null,
                "tcp",
                null,
                requireEnv("ROBOT_BUS_MESSAGE_XSUB"),
                requireEnv("ROBOT_BUS_MESSAGE_XPUB"),
                requireEnv("ROBOT_BUS_SERVICE_FRONTEND"),
                requireEnv("ROBOT_BUS_SERVICE_BACKEND"),
                requireEnv("ROBOT_BUS_ACTION_BACKEND"),
                requireEnv("ROBOT_BUS_ACTION_FRONTEND"));
    }

    private static String requireEnv(String key) {
        String v = System.getenv(key);
        if (v == null || v.isEmpty()) {
            throw new IllegalStateException("missing env " + key);
        }
        return v;
    }
}
