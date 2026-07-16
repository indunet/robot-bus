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

pub(crate) struct Timer {
    pub id: u64,
    pub period: Duration,
    pub next_deadline: Instant,
    pub callback: TimerCallback,
    pub group: CallbackGroup,
    pub cancelled: bool,
}

impl Timer {
    pub fn new(
        id: u64,
        period: Duration,
        callback: TimerCallback,
        group: CallbackGroup,
    ) -> Self {
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
pub(crate) fn ms_until_next_timer(timers: &[Timer], now: Instant) -> Option<i64> {
    timers
        .iter()
        .filter(|t| !t.cancelled)
        .map(|t| {
            if t.next_deadline <= now {
                0
            } else {
                t.next_deadline
                    .duration_since(now)
                    .as_millis()
                    .min(i64::MAX as u128) as i64
            }
        })
        .min()
}

pub(crate) fn effective_poll_timeout_ms(
    timers: &[Timer],
    requested_ms: i64,
    now: Instant,
) -> i64 {
    match ms_until_next_timer(timers, now) {
        Some(until) => until.min(requested_ms.max(0)),
        None => requested_ms.max(0),
    }
}
