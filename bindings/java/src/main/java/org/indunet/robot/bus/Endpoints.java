package org.indunet.robot.bus;

/** Resolves default message broker endpoints. */
public final class Endpoints {
    private Endpoints() {}

    public static String messageXsubEndpoint() {
        return messageXsubEndpoint("localhost", "tcp");
    }

    public static String messageXsubEndpoint(String host) {
        return messageXsubEndpoint(host, "tcp");
    }

    public static String messageXsubEndpoint(String host, String transport) {
        return NativeUtils.endpointCall(
                out -> RobotBusC.Holder.INSTANCE.robot_bus_message_xsub_endpoint(host, transport, out));
    }

    public static String messageXpubEndpoint() {
        return messageXpubEndpoint("localhost", "tcp");
    }

    public static String messageXpubEndpoint(String host) {
        return messageXpubEndpoint(host, "tcp");
    }

    public static String messageXpubEndpoint(String host, String transport) {
        return NativeUtils.endpointCall(
                out -> RobotBusC.Holder.INSTANCE.robot_bus_message_xpub_endpoint(host, transport, out));
    }
}
