//! Periodic timers for [`super::Executor`] (ROS 2–style `create_timer`).

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::runtime::callback_group::CallbackGroup;
use crate::runtime::worker_pool::WorkerPool;

pub type TimerCallback = Arc<dyn Fn() + Send + Sync>;

/// Opaque id returned by [`super::Executor::create_timer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerHandle {
    pub(crate) id: u64,
}

/// Opaque id returned by [`super::Executor::subscribe`] / Node create_subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionHandle {
    pub(crate) id: u64,
}

impl SubscriptionHandle {
    pub fn id(self) -> u64 {
        self.id
    }
}

pub(crate) struct Timer {
    pub id: u64,
    pub period: Duration,
    pub next_deadline: Instant,
    pub callback: TimerCallback,
    pub group: CallbackGroup,
    pub cancelled: bool,
}

impl Timer {
    pub fn new(id: u64, period: Duration, callback: TimerCallback, group: CallbackGroup) -> Self {
        Self {
            id,
            period,
            next_deadline: Instant::now() + period,
            callback,
            group,
            cancelled: false,
        }
    }
}

/// Fire every due timer once, then reschedule from `now + period`.
///
/// Returns `true` if at least one callback was scheduled/ran.
pub(crate) fn tick_timers(
    timers: &mut [Timer],
    now: Instant,
    worker_pool: Option<&WorkerPool>,
) -> bool {
    let mut fired = false;
    for timer in timers.iter_mut() {
        if timer.cancelled || timer.next_deadline > now {
            continue;
        }
        let callback = Arc::clone(&timer.callback);
        let group = timer.group.clone();
        group.run(worker_pool, move || callback());
        timer.next_deadline = now + timer.period;
        fired = true;
    }
    fired
}

/// Milliseconds until the soonest active timer, or `None` if none.
///
/// Rounds up: a sub-millisecond remainder rounded to 0 would make the poll
/// return immediately and spin until the deadline.
pub(crate) fn ms_until_next_timer(timers: &[Timer], now: Instant) -> Option<i64> {
    timers
        .iter()
        .filter(|t| !t.cancelled)
        .map(|t| {
            t.next_deadline
                .saturating_duration_since(now)
                .as_nanos()
                .div_ceil(1_000_000)
                .min(i64::MAX as u128) as i64
        })
        .min()
}

pub(crate) fn effective_poll_timeout_ms(timers: &[Timer], requested_ms: i64, now: Instant) -> i64 {
    match ms_until_next_timer(timers, now) {
        Some(until) => until.min(requested_ms.max(0)),
        None => requested_ms.max(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timer_due_in(now: Instant, remaining: Duration) -> Timer {
        let mut timer = Timer::new(
            1,
            Duration::from_secs(1),
            Arc::new(|| {}),
            CallbackGroup::mutually_exclusive(),
        );
        timer.next_deadline = now + remaining;
        timer
    }

    #[test]
    fn sub_millisecond_remainder_waits_one_millisecond() {
        let now = Instant::now();
        let timers = [timer_due_in(now, Duration::from_micros(300))];
        assert_eq!(ms_until_next_timer(&timers, now), Some(1));
    }

    #[test]
    fn partial_milliseconds_round_up() {
        let now = Instant::now();
        let timers = [timer_due_in(now, Duration::from_micros(2_100))];
        assert_eq!(ms_until_next_timer(&timers, now), Some(3));
    }

    #[test]
    fn overdue_timer_does_not_wait() {
        let now = Instant::now();
        let mut timer = timer_due_in(now, Duration::ZERO);
        timer.next_deadline = now - Duration::from_millis(5);
        assert_eq!(ms_until_next_timer(&[timer], now), Some(0));
    }
}
