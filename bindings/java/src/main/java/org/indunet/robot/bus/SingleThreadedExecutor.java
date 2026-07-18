package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Single-threaded executor that can own and spin one or more nodes. */
public final class SingleThreadedExecutor implements AutoCloseable {
    private Pointer ptr;

    public SingleThreadedExecutor() {
        this.ptr =
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_new(),
                        "SingleThreadedExecutor");
    }

    public void addNode(Node node) {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_add_node(ptr, node.raw()),
                "add_node");
    }

    public Node createNode(String name) {
        return createNode(name, null);
    }

    public Node createNode(String name, NodeOptions options) {
        return Node.fromRaw(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_create_node(
                                ptr, name, options != null ? options.toNative() : null),
                        "create_node"));
    }

    public ShutdownHandle shutdownHandle() {
        return new ShutdownHandle(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_shutdown_handle(ptr),
                        "shutdown_handle"));
    }

    public void shutdown() {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_shutdown(ptr), "shutdown");
    }

    public boolean spinOnce() {
        return spinOnce(-1.0);
    }

    public boolean spinOnce(double timeoutSecs) {
        int rc = RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_spin_once(ptr, timeoutSecs);
        if (rc < 0) {
            Errors.check(rc, "spin_once");
        }
        return rc == 1;
    }

    public void spin() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_spin(ptr), "spin");
    }

    public void start() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_start(ptr), "start");
    }

    public void stop() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_stop(ptr), "stop");
    }

    public void waitForShutdown() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_wait(ptr), "wait");
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_single_threaded_executor_free(ptr);
        ptr = null;
    }
}
