//! Bounded concurrency for service/action handlers (simple MultiThreadedExecutor).

use std::sync::{Arc, Condvar, Mutex};

/// Limits how many handler threads may run at once.
///
/// If no slot is free, the caller should run the handler inline on the I/O
/// thread instead of blocking the poll loop.
#[derive(Clone)]
pub struct WorkerPool {
    inner: Arc<(Mutex<usize>, Condvar)>,
}

impl WorkerPool {
    pub fn new(max_workers: usize) -> Self {
        let max_workers = max_workers.max(1);
        Self {
            inner: Arc::new((Mutex::new(max_workers), Condvar::new())),
        }
    }

    /// Non-blocking: returns `false` when all slots are busy.
    pub fn try_acquire(&self) -> bool {
        let (lock, _) = &*self.inner;
        let mut available = lock.lock().expect("worker pool mutex");
        if *available == 0 {
            return false;
        }
        *available -= 1;
        true
    }

    pub fn release(&self) {
        let (lock, cvar) = &*self.inner;
        let mut available = lock.lock().expect("worker pool mutex");
        *available += 1;
        cvar.notify_one();
    }
}
