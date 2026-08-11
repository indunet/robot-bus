package org.indunet.robot.bus;

/** HTTP discovery options (maps to C {@code RobotBusDiscoverOpts}: GET /api/v1/discover). */
public final class DiscoverOpts {
    private final String apiUrl;
    private final String brokerId;
    private final double timeoutSecs;

    public DiscoverOpts() {
        this(null, null, 0.0);
    }

    /** @param apiUrl broker API base, e.g. {@code http://127.0.0.1:15570}; null = default */
    public DiscoverOpts(String apiUrl) {
        this(apiUrl, null, 0.0);
    }

    public DiscoverOpts(String apiUrl, String brokerId, double timeoutSecs) {
        this.apiUrl = apiUrl;
        this.brokerId = brokerId;
        this.timeoutSecs = timeoutSecs;
    }

    public String getApiUrl() {
        return apiUrl;
    }

    public String getBrokerId() {
        return brokerId;
    }

    public double getTimeoutSecs() {
        return timeoutSecs;
    }

    RobotBusC.DiscoverOpts toNative() {
        RobotBusC.DiscoverOpts o = new RobotBusC.DiscoverOpts();
        o.apiUrl = apiUrl;
        o.brokerId = brokerId;
        o.timeoutSecs = timeoutSecs;
        o.write();
        return o;
    }
}
