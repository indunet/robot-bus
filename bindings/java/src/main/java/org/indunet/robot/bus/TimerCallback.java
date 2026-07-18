package org.indunet.robot.bus;

/** Callback for timer firings. */
@FunctionalInterface
public interface TimerCallback {
    void onTimer();
}
