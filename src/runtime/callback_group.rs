//! ROS 2–style callback groups: mutually exclusive vs reentrant.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::runtime::worker_pool::WorkerPool;

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
        Self {
            id: NEXT_GROUP_ID.fetch_add(1, Ordering::Relaxed),
            kind,
            exclusive: Arc::new(Mutex::new(ExclusiveState {
                running: false,
                pending: VecDeque::new(),
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
        match self.kind {
            CallbackGroupType::MutuallyExclusive => {
                if let Some(pool) = worker_pool {
                    submit_exclusive(pool, &self.exclusive, Box::new(job));
                    return;
                }
                job();
            }
            CallbackGroupType::Reentrant => {
                if let Some(pool) = worker_pool {
                    pool.submit(job);
                    return;
                }
                job();
            }
        }
    }
}

fn submit_exclusive(pool: &WorkerPool, exclusive: &Arc<Mutex<ExclusiveState>>, job: Job) {
    {
        let mut state = exclusive.lock().expect("callback group mutex");
        if state.running {
            state.pending.push_back(job);
            return;
        }
        state.running = true;
    }
    dispatch_exclusive(pool, Arc::clone(exclusive), job);
}

fn dispatch_exclusive(pool: &WorkerPool, exclusive: Arc<Mutex<ExclusiveState>>, job: Job) {
    let pool_for_next = pool.clone();
    pool.submit(move || {
        job();
        let next = {
            let mut state = exclusive.lock().expect("callback group mutex");
            match state.pending.pop_front() {
                Some(next) => Some(next),
                None => {
                    state.running = false;
                    None
                }
            }
        };
        if let Some(next) = next {
            dispatch_exclusive(&pool_for_next, exclusive, next);
        }
    });
}

/// Topic subscription entry: user callback + callback group.
#[derive(Clone)]
pub struct SubscriptionCallback {
    pub id: u64,
    pub callback: crate::runtime::registrations::MessageCallback,
    pub group: CallbackGroup,
}
