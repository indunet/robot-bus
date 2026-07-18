package org.indunet.robot.bus;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.indunet.robot.bus.geometry_msgs.msg.v1.Vector3;
import org.indunet.robot.bus.robot_bus_interface.action.v1.FibonacciGoal;
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolRequest;
import org.indunet.robot.bus.std_srvs.srv.v1.SetBoolResponse;
import org.junit.jupiter.api.Test;

/** Pure protobuf serialize/parse (no broker) for generated Java msgs. */
class MsgsRoundtripTest {
    @Test
    void imuRoundtrip() throws Exception {
        Imu imu =
                Imu.newBuilder()
                        .setAngularVelocity(Vector3.newBuilder().setZ(0.1).build())
                        .setLinearAcceleration(Vector3.newBuilder().setZ(9.8).build())
                        .build();
        byte[] bytes = imu.toByteArray();
        Imu parsed = Imu.parseFrom(bytes);
        assertEquals(0.1, parsed.getAngularVelocity().getZ(), 1e-9);
        assertEquals(9.8, parsed.getLinearAcceleration().getZ(), 1e-9);
    }

    @Test
    void setBoolRoundtrip() throws Exception {
        SetBoolRequest req = SetBoolRequest.newBuilder().setData(true).build();
        SetBoolRequest parsed = SetBoolRequest.parseFrom(req.toByteArray());
        assertTrue(parsed.getData());

        SetBoolResponse resp =
                SetBoolResponse.newBuilder().setSuccess(true).setMessage("ok").build();
        assertEquals("ok", SetBoolResponse.parseFrom(resp.toByteArray()).getMessage());
    }

    @Test
    void fibonacciGoalRoundtrip() throws Exception {
        FibonacciGoal goal = FibonacciGoal.newBuilder().setOrder(5).build();
        assertEquals(5, FibonacciGoal.parseFrom(goal.toByteArray()).getOrder());
    }
}