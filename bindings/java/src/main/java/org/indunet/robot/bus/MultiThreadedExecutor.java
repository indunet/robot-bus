package org.indunet.robot.bus;

import com.sun.jna.Pointer;

/** Multi-threaded executor that can own and spin one or more nodes. */
public final class MultiThreadedExecutor implements AutoCloseable {
    private Pointer ptr;

    public MultiThreadedExecutor() {
        this(0);
    }

    public MultiThreadedExecutor(int numThreads) {
        this.ptr =
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_new(numThreads),
                        "MultiThreadedExecutor");
    }

    public MultiThreadedExecutor(Context context) {
        this(context, 0);
    }

    public MultiThreadedExecutor(Context context, int numThreads) {
        this.ptr =
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_new_with_context(
                                context.raw(), numThreads),
                        "MultiThreadedExecutor");
    }

    public void addNode(Node node) {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_add_node(ptr, node.raw()),
                "add_node");
    }

    public Node createNode(String name) {
        return createNode(name, null);
    }

    public Node createNode(String name, NodeOptions options) {
        return Node.fromRaw(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_create_node(
                                ptr, name, options != null ? options.toNative() : null),
                        "create_node"));
    }

    public ShutdownHandle shutdownHandle() {
        return new ShutdownHandle(
                Errors.checkPtr(
                        RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_shutdown_handle(ptr),
                        "shutdown_handle"));
    }

    public void shutdown() {
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_shutdown(ptr), "shutdown");
    }

    public boolean spinOnce() {
        return spinOnce(-1.0);
    }

    public boolean spinOnce(double timeoutSecs) {
        int rc = RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_spin_once(ptr, timeoutSecs);
        if (rc < 0) {
            Errors.check(rc, "spin_once");
        }
        return rc == 1;
    }

    public void spin() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_spin(ptr), "spin");
    }

    @Override
    public void close() {
        RobotBusC.Holder.INSTANCE.robot_bus_multi_threaded_executor_free(ptr);
        ptr = null;
    }
}
