//! Shared predicate for opt-in ROS2→bus topic lazy subscribe.

/// Whether a ROS 2 subscription should exist for a bridged topic.
///
/// * Eager routes (`lazy == false`) are always on.
/// * Lazy routes wait until the broker console is known: live console honors
///   `subscribers > 0`; no console degrades to eager so traffic still flows.
/// * `console_live == None` means we have not seen a console snapshot yet —
///   keep the ROS subscription off so the graph does not flash at startup.
pub fn should_enable_ros_subscription(
    lazy: bool,
    console_live: Option<bool>,
    subscribers: u32,
) -> bool {
    if !lazy {
        return true;
    }
    match console_live {
        Some(false) => true,
        Some(true) => subscribers > 0,
        None => false,
    }
}

/// How long a lazy bridge waits for `/robot_bus/topics` or `/robot_bus/topic_demand`
/// before treating the broker as `--no-console` and falling back to eager.
///
/// Longer than the console's 1 Hz snapshot so a live console is not mistaken
/// for `--no-console` during startup.
pub const CONSOLE_DETECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

#[cfg(test)]
mod tests {
    use super::should_enable_ros_subscription;

    #[test]
    fn eager_always_on() {
        assert!(should_enable_ros_subscription(false, None, 0));
        assert!(should_enable_ros_subscription(false, Some(true), 0));
        assert!(should_enable_ros_subscription(false, Some(false), 0));
    }

    #[test]
    fn lazy_honors_subscriber_count_when_console_live() {
        assert!(!should_enable_ros_subscription(true, Some(true), 0));
        assert!(should_enable_ros_subscription(true, Some(true), 1));
        assert!(should_enable_ros_subscription(true, Some(true), 2));
    }

    #[test]
    fn lazy_without_console_falls_back_eager() {
        assert!(should_enable_ros_subscription(true, Some(false), 0));
    }

    #[test]
    fn lazy_stays_off_until_console_known() {
        assert!(!should_enable_ros_subscription(true, None, 0));
        assert!(!should_enable_ros_subscription(true, None, 5));
    }
}
