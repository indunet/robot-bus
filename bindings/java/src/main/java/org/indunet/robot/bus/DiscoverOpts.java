package org.indunet.robot.bus;

/** UDP multicast discovery options (maps to C {@code RobotBusDiscoverOpts}). */
public final class DiscoverOpts {
    private final int domainId;
    private final String brokerId;
    private final String multicastAddr;
    private final int multicastPort;
    private final double timeoutSecs;

    public DiscoverOpts() {
        this(0, null, null, 0, 0.0);
    }

    public DiscoverOpts(int domainId) {
        this(domainId, null, null, 0, 0.0);
    }

    public DiscoverOpts(
            int domainId,
            String brokerId,
            String multicastAddr,
            int multicastPort,
            double timeoutSecs) {
        this.domainId = domainId;
        this.brokerId = brokerId;
        this.multicastAddr = multicastAddr;
        this.multicastPort = multicastPort;
        this.timeoutSecs = timeoutSecs;
    }

    public int getDomainId() {
        return domainId;
    }

    public String getBrokerId() {
        return brokerId;
    }

    public String getMulticastAddr() {
        return multicastAddr;
    }

    public int getMulticastPort() {
        return multicastPort;
    }

    public double getTimeoutSecs() {
        return timeoutSecs;
    }

    RobotBusC.DiscoverOpts toNative() {
        RobotBusC.DiscoverOpts o = new RobotBusC.DiscoverOpts();
        o.domainId = domainId;
        o.brokerId = brokerId;
        o.multicastAddr = multicastAddr;
        o.multicastPort = (short) (multicastPort & 0xffff);
        o.timeoutSecs = timeoutSecs;
        o.write();
        return o;
    }
}
