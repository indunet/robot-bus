//! Resident worker threads for [`crate::runtime::MultiThreadedExecutor`].

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Default number of waiting jobs (running jobs do not count toward this limit).
pub const DEFAULT_QUEUE_CAPACITY: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SubmitError {
    #[error("callback queue is full")]
    Full,
    #[error("worker pool is closed")]
    Closed,
}

/// Snapshot of a callback queue. Counters are local to this pool or group.
#[derive(Debug, Clone, Copy)]
pub struct QueueStats {
    /// Total slots, including the pool's internal continuation reserve.
    pub capacity: usize,
    pub pending: usize,
    pub rejected: u64,
}

#[derive(Default)]
struct Counters {
    pending: AtomicUsize,
    rejected: AtomicU64,
}

/// Isolate user panics without losing a resident worker or group bookkeeping.
pub(crate) fn run_callback<F, T>(callback: F) -> std::thread::Result<T>
where
    F: FnOnce() -> T,
{
    let result = catch_unwind(AssertUnwindSafe(callback));
    if result.is_err() {
        log::error!("robot-bus callback panicked");
    }
    result
}

struct WorkerPoolInner {
    tx: Mutex<Option<SyncSender<Job>>>,
    capacity: usize,
    admission_limit: usize,
    counters: Arc<Counters>,
    threads: Mutex<Vec<JoinHandle<()>>>,
}

impl Drop for WorkerPoolInner {
    fn drop(&mut self) {
        {
            let mut tx = self.tx.lock().expect("worker pool send mutex");
            *tx = None;
        }
        let threads = {
            let mut guard = self.threads.lock().expect("worker pool thread mutex");
            std::mem::take(&mut *guard)
        };
        for handle in threads {
            // A submitted job may own the final pool handle. Never join ourselves.
            if handle.thread().id() != thread::current().id() {
                let _ = handle.join();
            }
        }
    }
}

/// Fixed-size thread pool with a bounded waiting queue. Full queues reject new jobs.
///
/// [`try_submit`](Self::try_submit) never spawns a thread and never runs the job on the
/// caller. Drop disconnects the queue and joins all workers.
#[derive(Clone)]
pub struct WorkerPool {
    inner: Arc<WorkerPoolInner>,
}

#[derive(Clone)]
pub(crate) struct WeakWorkerPool(Weak<WorkerPoolInner>);

impl WeakWorkerPool {
    pub(crate) fn upgrade(&self) -> Option<WorkerPool> {
        self.0.upgrade().map(|inner| WorkerPool { inner })
    }
}

impl WorkerPool {
    pub fn new(max_workers: usize) -> Self {
        Self::with_queue_capacity(max_workers, DEFAULT_QUEUE_CAPACITY)
    }

    /// Set the waiting-job limit. Zero is clamped to one.
    pub fn with_queue_capacity(max_workers: usize, queue_capacity: usize) -> Self {
        let n = max_workers.max(1);
        let admission_limit = queue_capacity.max(1);
        // A running worker can yield its group's continuation even when all
        // external queue slots are occupied. At most n such slots are needed.
        let capacity = admission_limit.saturating_add(n);
        let (tx, rx) = mpsc::sync_channel::<Job>(capacity);
        let rx = Arc::new(Mutex::new(rx));
        let mut threads = Vec::with_capacity(n);
        for i in 0..n {
            let rx = Arc::clone(&rx);
            let handle = thread::Builder::new()
                .name(format!("robot-bus-worker-{i}"))
                .spawn(move || worker_loop(&rx))
                .expect("spawn worker pool thread");
            threads.push(handle);
        }
        Self {
            inner: Arc::new(WorkerPoolInner {
                tx: Mutex::new(Some(tx)),
                capacity,
                admission_limit,
                counters: Arc::new(Counters::default()),
                threads: Mutex::new(threads),
            }),
        }
    }

    pub(crate) fn downgrade(&self) -> WeakWorkerPool {
        WeakWorkerPool(Arc::downgrade(&self.inner))
    }

    pub fn queue_stats(&self) -> QueueStats {
        QueueStats {
            capacity: self.inner.capacity,
            pending: self.inner.counters.pending.load(Ordering::Relaxed),
            rejected: self.inner.counters.rejected.load(Ordering::Relaxed),
        }
    }

    /// Try to enqueue a job, returning immediately when the queue is full.
    pub fn try_submit<F>(&self, job: F) -> Result<(), SubmitError>
    where
        F: FnOnce() + Send + 'static,
    {
        self.enqueue(job, false)
    }

    pub(crate) fn submit_continuation<F>(&self, job: F) -> Result<(), SubmitError>
    where
        F: FnOnce() + Send + 'static,
    {
        self.enqueue(job, true)
    }

    fn enqueue<F>(&self, job: F, continuation: bool) -> Result<(), SubmitError>
    where
        F: FnOnce() + Send + 'static,
    {
        let counters = Arc::clone(&self.inner.counters);
        let job_counters = Arc::clone(&counters);
        let job: Job = Box::new(move || {
            job_counters.pending.fetch_sub(1, Ordering::Relaxed);
            job();
        });
        // Drop rejected closures only after releasing the sender lock: their
        // captures can include pool handles or other objects with custom Drop.
        let result = {
            let guard = self.inner.tx.lock().expect("worker pool send mutex");
            if !continuation
                && counters.pending.load(Ordering::Relaxed) >= self.inner.admission_limit
            {
                counters.rejected.fetch_add(1, Ordering::Relaxed);
                drop(guard);
                return Err(SubmitError::Full);
            }
            counters.pending.fetch_add(1, Ordering::Relaxed);
            match guard.as_ref() {
                Some(tx) => tx.try_send(job),
                None => Err(mpsc::TrySendError::Disconnected(job)),
            }
        };
        match result {
            Ok(()) => Ok(()),
            Err(err) => {
                counters.pending.fetch_sub(1, Ordering::Relaxed);
                if !continuation {
                    counters.rejected.fetch_add(1, Ordering::Relaxed);
                }
                Err(match err {
                    mpsc::TrySendError::Full(_) => SubmitError::Full,
                    mpsc::TrySendError::Disconnected(_) => SubmitError::Closed,
                })
            }
        }
    }
}

fn worker_loop(rx: &Mutex<Receiver<Job>>) {
    loop {
        let job = {
            let guard = rx.lock().expect("worker pool recv mutex");
            guard.recv()
        };
        match job {
            Ok(job) => {
                let _ = run_callback(job);
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    #[test]
    fn submit_runs_and_drop_joins() {
        let pool = WorkerPool::new(2);
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_job = Arc::clone(&hits);
        pool.try_submit(move || {
            hits_job.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while hits.load(Ordering::SeqCst) == 0 {
            assert!(Instant::now() < deadline, "worker did not run job");
            thread::sleep(Duration::from_millis(1));
        }
        drop(pool);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn panic_does_not_remove_a_worker() {
        let pool = WorkerPool::new(1);
        pool.try_submit(|| panic!("user callback")).unwrap();
        let (tx, rx) = mpsc::channel();
        pool.try_submit(move || tx.send(()).unwrap()).unwrap();
        rx.recv_timeout(Duration::from_secs(2))
            .expect("worker must survive panic");
    }

    #[test]
    fn full_queue_rejects_without_blocking_and_recovers() {
        let pool = WorkerPool::with_queue_capacity(1, 1);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        pool.try_submit(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        })
        .unwrap();
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        pool.try_submit(move || done_tx.send(()).unwrap()).unwrap();
        let result = pool.try_submit(|| panic!("rejected job must never run"));
        let stats = pool.queue_stats();
        release_tx.send(()).unwrap();
        assert_eq!(result, Err(SubmitError::Full));
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.rejected, 1);
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(pool.queue_stats().pending, 0);
    }
}
