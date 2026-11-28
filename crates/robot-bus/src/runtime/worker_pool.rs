//! Resident worker threads for [`crate::runtime::MultiThreadedExecutor`].

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;
type CompletionHook = Arc<dyn Fn() + Send + Sync>;

struct CompletionNotify(Option<CompletionHook>);

impl Drop for CompletionNotify {
    fn drop(&mut self) {
        if let Some(hook) = self.0.take() {
            if catch_unwind(AssertUnwindSafe(|| hook())).is_err() {
                log::error!("robot-bus worker completion hook panicked");
            }
        }
    }
}

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
    tx: crossbeam_channel::Sender<Job>,
    capacity: usize,
    admission_limit: usize,
    worker_count: usize,
    counters: Arc<Counters>,
    completion: Mutex<Option<CompletionHook>>,
    threads: Mutex<Vec<JoinHandle<()>>>,
}

impl Drop for WorkerPoolInner {
    fn drop(&mut self) {
        let (disconnected, _) = crossbeam_channel::bounded::<Job>(1);
        drop(std::mem::replace(&mut self.tx, disconnected));
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
        let (tx, rx) = crossbeam_channel::bounded::<Job>(capacity);
        let mut threads = Vec::with_capacity(n);
        for i in 0..n {
            let rx = rx.clone();
            let handle = thread::Builder::new()
                .name(format!("robot-bus-worker-{i}"))
                .spawn(move || worker_loop(rx))
                .expect("spawn worker pool thread");
            threads.push(handle);
        }
        drop(rx);
        Self {
            inner: Arc::new(WorkerPoolInner {
                tx,
                capacity,
                admission_limit,
                worker_count: n,
                counters: Arc::new(Counters::default()),
                completion: Mutex::new(None),
                threads: Mutex::new(threads),
            }),
        }
    }

    pub(crate) fn worker_count(&self) -> usize {
        self.inner.worker_count
    }

    /// Runs after each job, including jobs that panic. Installed once by the executor.
    pub(crate) fn set_completion_hook<F>(&self, hook: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        *self.inner.completion.lock().expect("completion hook") = Some(Arc::new(hook));
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
        if !continuation && counters.pending.load(Ordering::Relaxed) >= self.inner.admission_limit {
            counters.rejected.fetch_add(1, Ordering::Relaxed);
            return Err(SubmitError::Full);
        }
        let job_counters = Arc::clone(&counters);
        let hook = self
            .inner
            .completion
            .lock()
            .expect("completion hook")
            .clone();
        let job: Job = Box::new(move || {
            job_counters.pending.fetch_sub(1, Ordering::Relaxed);
            let _notify = CompletionNotify(hook);
            job();
        });
        counters.pending.fetch_add(1, Ordering::Relaxed);
        match self.inner.tx.try_send(job) {
            Ok(()) => Ok(()),
            Err(err) => {
                counters.pending.fetch_sub(1, Ordering::Relaxed);
                if !continuation {
                    counters.rejected.fetch_add(1, Ordering::Relaxed);
                }
                Err(match err {
                    crossbeam_channel::TrySendError::Full(_) => SubmitError::Full,
                    crossbeam_channel::TrySendError::Disconnected(_) => SubmitError::Closed,
                })
            }
        }
    }
}

fn worker_loop(rx: crossbeam_channel::Receiver<Job>) {
    while let Ok(job) = rx.recv() {
        let _ = run_callback(job);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
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
    fn completion_hook_runs_after_success_and_panic() {
        let pool = WorkerPool::new(1);
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_hook = Arc::clone(&hits);
        pool.set_completion_hook(move || {
            hits_hook.fetch_add(1, Ordering::SeqCst);
        });
        pool.try_submit(|| {}).unwrap();
        pool.try_submit(|| panic!("hook must still run")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while hits.load(Ordering::SeqCst) < 2 {
            assert!(Instant::now() < deadline, "completion hook did not run");
            thread::sleep(Duration::from_millis(1));
        }
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
