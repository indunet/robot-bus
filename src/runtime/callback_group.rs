//! ROS 2–style callback groups: mutually exclusive vs reentrant.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

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

/// Groups callbacks for concurrency control (ROS 2 `CallbackGroup`).
#[derive(Clone)]
pub struct CallbackGroup {
    id: u64,
    kind: CallbackGroupType,
    exclusive: Arc<Mutex<()>>,
}

impl CallbackGroup {
    pub fn new(kind: CallbackGroupType) -> Self {
        Self {
            id: NEXT_GROUP_ID.fetch_add(1, Ordering::Relaxed),
            kind,
            exclusive: Arc::new(Mutex::new(())),
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
    pub(crate) fn run<F>(&self, worker_pool: Option<&WorkerPool>, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self.kind {
            CallbackGroupType::MutuallyExclusive => {
                if let Some(pool) = worker_pool {
                    if pool.try_acquire() {
                        let exclusive = Arc::clone(&self.exclusive);
                        let pool = pool.clone();
                        thread::spawn(move || {
                            let _guard = exclusive.lock().expect("callback group mutex");
                            job();
                            pool.release();
                        });
                        return;
                    }
                }
                let _guard = self.exclusive.lock().expect("callback group mutex");
                job();
            }
            CallbackGroupType::Reentrant => {
                if let Some(pool) = worker_pool {
                    if pool.try_acquire() {
                        let pool = pool.clone();
                        thread::spawn(move || {
                            job();
                            pool.release();
                        });
                        return;
                    }
                }
                job();
            }
        }
    }
}

/// Topic subscription entry: user callback + callback group.
#[derive(Clone)]
pub struct SubscriptionCallback {
    pub id: u64,
    pub callback: crate::runtime::registrations::MessageCallback,
    pub group: CallbackGroup,
}
