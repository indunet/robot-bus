//! ROS 2–style callback groups: mutually exclusive vs reentrant.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::runtime::worker_pool::{
    DEFAULT_QUEUE_CAPACITY, QueueStats, SubmitError, WeakWorkerPool, WorkerPool, run_callback,
};

/// How callbacks in a [`CallbackGroup`] may overlap (ROS 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackGroupType {
    /// At most one callback from this group runs at a time.
    MutuallyExclusive,
    /// Callbacks from this group may run concurrently (with a multi-threaded executor).
    Reentrant,
}

static NEXT_GROUP_ID: AtomicU64 = AtomicU64::new(1);

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ExclusiveState {
    running: bool,
    pending: VecDeque<Job>,
    capacity: usize,
    rejected: u64,
}

/// Groups callbacks for concurrency control (ROS 2 `CallbackGroup`).
#[derive(Clone)]
pub struct CallbackGroup {
    id: u64,
    kind: CallbackGroupType,
    exclusive: Arc<Mutex<ExclusiveState>>,
}

impl CallbackGroup {
    pub fn new(kind: CallbackGroupType) -> Self {
        Self::with_queue_capacity(kind, DEFAULT_QUEUE_CAPACITY)
    }

    /// Set the mutually exclusive group's waiting-job limit (at least one).
    /// Reentrant groups use the worker pool's queue limit instead.
    pub fn with_queue_capacity(kind: CallbackGroupType, capacity: usize) -> Self {
        Self {
            id: NEXT_GROUP_ID.fetch_add(1, Ordering::Relaxed),
            kind,
            exclusive: Arc::new(Mutex::new(ExclusiveState {
                running: false,
                pending: VecDeque::new(),
                capacity: capacity.max(1),
                rejected: 0,
            })),
        }
    }

    pub fn mutually_exclusive() -> Self {
        Self::new(CallbackGroupType::MutuallyExclusive)
    }

    pub fn reentrant() -> Self {
        Self::new(CallbackGroupType::Reentrant)
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn kind(&self) -> CallbackGroupType {
        self.kind
    }

    /// Run `job`, optionally on the worker pool, respecting this group's type.
    ///
    /// With a pool, jobs are never run on the caller: they are queued. A
    /// mutually exclusive group keeps at most one job in flight (extra work
    /// waits on a per-group pending queue). Without a pool, `job` runs inline
    /// on the poll / I/O thread.
    pub(crate) fn run<F>(&self, worker_pool: Option<&WorkerPool>, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Err(err) = self.try_run(worker_pool, job) {
            log::warn!("callback group {}: {err}", self.id);
        }
    }

    /// Local waiting queue statistics (reentrant work lives in the pool).
    pub fn queue_stats(&self) -> QueueStats {
        let state = self.exclusive.lock().expect("callback group mutex");
        QueueStats {
            capacity: state.capacity,
            pending: state.pending.len(),
            rejected: state.rejected,
        }
    }

    pub(crate) fn try_run<F>(
        &self,
        worker_pool: Option<&WorkerPool>,
        job: F,
    ) -> Result<(), SubmitError>
    where
        F: FnOnce() + Send + 'static,
    {
        match (self.kind, worker_pool) {
            (CallbackGroupType::MutuallyExclusive, Some(pool)) => {
                submit_exclusive(pool, &self.exclusive, Box::new(job))
            }
            (CallbackGroupType::Reentrant, Some(pool)) => pool.try_submit(job),
            (_, None) => {
                let _ = run_callback(job);
                Ok(())
            }
        }
    }
}

fn submit_exclusive(
    pool: &WorkerPool,
    exclusive: &Arc<Mutex<ExclusiveState>>,
    job: Job,
) -> Result<(), SubmitError> {
    let mut state = exclusive.lock().expect("callback group mutex");
    if state.running {
        if state.pending.len() >= state.capacity {
            state.rejected += 1;
            drop(state);
            return Err(SubmitError::Full);
        }
        state.pending.push_back(job);
        return Ok(());
    }
    state.running = true;
    let runner_state = Arc::clone(exclusive);
    let pool_for_next = pool.downgrade();
    let result = pool.try_submit(move || run_exclusive(pool_for_next, runner_state, Some(job)));
    if result.is_err() {
        state.running = false;
        state.rejected += 1;
    }
    result
}

fn run_exclusive(
    pool: WeakWorkerPool,
    exclusive: Arc<Mutex<ExclusiveState>>,
    mut current: Option<Job>,
) {
    loop {
        if let Some(job) = current.take() {
            let _ = run_callback(job);
        }
        {
            let mut state = exclusive.lock().expect("callback group mutex");
            if state.pending.is_empty() {
                state.running = false;
                return;
            }
        }
        // Yield after each callback so a busy group cannot starve other groups.
        // The next callback stays in the group queue until its runner executes.
        if let Some(resident_pool) = pool.upgrade() {
            let next_pool = pool.clone();
            let next_state = Arc::clone(&exclusive);
            if resident_pool
                .submit_continuation(move || {
                    let next = next_state
                        .lock()
                        .expect("callback group mutex")
                        .pending
                        .pop_front();
                    run_exclusive(next_pool, next_state, next);
                })
                .is_ok()
            {
                return;
            }
        }
        // The external pool owner is shutting down. Drain already accepted work
        // on this worker rather than losing jobs or resurrecting the pool.
        current = exclusive
            .lock()
            .expect("callback group mutex")
            .pending
            .pop_front();
    }
}

/// Topic subscription entry: user callback + callback group.
#[derive(Clone)]
pub struct SubscriptionCallback {
    pub id: u64,
    pub callback: crate::runtime::registrations::MessageCallback,
    pub group: CallbackGroup,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn panic_releases_exclusive_group_and_preserves_queued_work() {
        let pool = WorkerPool::new(1);
        let group = CallbackGroup::mutually_exclusive();
        let (start_tx, start_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || {
                start_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                panic!("user callback");
            })
            .unwrap();
        start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (next_tx, next_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || next_tx.send(()).unwrap())
            .unwrap();
        release_tx.send(()).unwrap();
        next_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("same group must recover");
        let (later_tx, later_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || later_tx.send(()).unwrap())
            .unwrap();
        later_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    }

    #[test]
    fn exclusive_pending_queue_is_bounded() {
        let pool = WorkerPool::new(1);
        let group = CallbackGroup::with_queue_capacity(CallbackGroupType::MutuallyExclusive, 1);
        let (start_tx, start_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || {
                start_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            })
            .unwrap();
        start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || done_tx.send(()).unwrap())
            .unwrap();
        let rejected = group.try_run(Some(&pool), || panic!("must not execute"));
        let stats = group.queue_stats();
        release_tx.send(()).unwrap();
        assert_eq!(rejected, Err(SubmitError::Full));
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.rejected, 1);
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    }

    #[test]
    fn full_pool_does_not_leave_group_running() {
        let pool = WorkerPool::with_queue_capacity(1, 1);
        let (start_tx, start_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        pool.try_submit(move || {
            start_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        })
        .unwrap();
        start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (drained_tx, drained_rx) = mpsc::channel();
        pool.try_submit(move || drained_tx.send(()).unwrap())
            .unwrap();
        let group = CallbackGroup::mutually_exclusive();
        let rejected = group.try_run(Some(&pool), || panic!("must not execute"));
        release_tx.send(()).unwrap();
        assert_eq!(rejected, Err(SubmitError::Full));
        drained_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || done_tx.send(()).unwrap())
            .unwrap();
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    #[test]
    fn busy_group_yields_even_when_external_pool_queue_is_full() {
        let pool = WorkerPool::with_queue_capacity(1, 1);
        let first = CallbackGroup::mutually_exclusive();
        let second = CallbackGroup::mutually_exclusive();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        first
            .try_run(Some(&pool), move || {
                started_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            })
            .unwrap();
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (order_tx, order_rx) = mpsc::channel();
        let next_tx = order_tx.clone();
        first
            .try_run(Some(&pool), move || {
                next_tx.send("first-group-next").unwrap()
            })
            .unwrap();
        second
            .try_run(Some(&pool), move || order_tx.send("other-group").unwrap())
            .unwrap();
        release_tx.send(()).unwrap();
        assert_eq!(
            order_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "other-group"
        );
        assert_eq!(
            order_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "first-group-next"
        );
        assert_eq!(pool.queue_stats().rejected, 0);
    }

    #[test]
    fn dropping_pool_drains_accepted_group_callbacks() {
        let pool = WorkerPool::new(1);
        let group = CallbackGroup::mutually_exclusive();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || {
                started_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            })
            .unwrap();
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        group
            .try_run(Some(&pool), move || done_tx.send(()).unwrap())
            .unwrap();
        release_tx.send(()).unwrap();
        drop(pool);
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(group.queue_stats().pending, 0);
    }
}
