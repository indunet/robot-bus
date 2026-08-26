//! Resident worker threads for [`crate::runtime::MultiThreadedExecutor`].

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct WorkerPoolInner {
    tx: Mutex<Option<Sender<Job>>>,
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
            let _ = handle.join();
        }
    }
}

/// Fixed-size thread pool: `n` resident workers pulling from an unbounded queue.
///
/// [`submit`](Self::submit) never spawns a thread and never runs the job on the
/// caller. Drop disconnects the queue and joins all workers.
#[derive(Clone)]
pub struct WorkerPool {
    inner: Arc<WorkerPoolInner>,
}

impl WorkerPool {
    pub fn new(max_workers: usize) -> Self {
        let n = max_workers.max(1);
        let (tx, rx) = mpsc::channel::<Job>();
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
                threads: Mutex::new(threads),
            }),
        }
    }

    /// Enqueue `job` for a resident worker. No-op after the pool has been shut down.
    pub fn submit<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let guard = self.inner.tx.lock().expect("worker pool send mutex");
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(Box::new(job));
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
            Ok(job) => job(),
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
        pool.submit(move || {
            hits_job.fetch_add(1, Ordering::SeqCst);
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while hits.load(Ordering::SeqCst) == 0 {
            assert!(Instant::now() < deadline, "worker did not run job");
            thread::sleep(Duration::from_millis(1));
        }
        drop(pool);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }
}
