package org.indunet.robot.bus;

/**
 * Process-wide hints set by {@code RobotBusAndroid.init}. Desktop JVM leaves these unset.
 */
final class AndroidRuntime {
    /** App cache directory used as the broker IPC root. Null off Android. */
    static volatile String ipcRoot;

    private AndroidRuntime() {}

    static void setIpcRoot(String root) {
        ipcRoot = root;
    }
}
