//! Callback group concurrency smoke tests.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::{CallbackGroupType, MultiThreadedExecutor, Node};

#[test]
fn mutually_exclusive_group_serializes_timers() {
    let overlapping = Arc::new(AtomicUsize::new(0));
    let max_seen = Arc::new(AtomicUsize::new(0));
    let overlapping_cb = overlapping.clone();
    let max_seen_cb = max_seen.clone();

    let mut node = Node::new("cg");
    let executor = MultiThreadedExecutor::new(4);
    executor.add_node(&mut node).expect("add_node");

    let group = node.create_callback_group(CallbackGroupType::MutuallyExclusive);
    for _ in 0..2 {
        let overlapping = overlapping_cb.clone();
        let max_seen = max_seen_cb.clone();
        node.create_timer(
            Duration::from_millis(20),
            Arc::new(move || {
                let now = overlapping.fetch_add(1, Ordering::SeqCst) + 1;
                max_seen.fetch_max(now, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(40));
                overlapping.fetch_sub(1, Ordering::SeqCst);
            }),
            Some(&group),
        )
        .expect("timer");
    }

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        executor
            .spin_once(Some(Duration::from_millis(10)))
            .expect("spin");
        if max_seen.load(Ordering::SeqCst) >= 1 {
            // Give concurrent attempt a chance if any.
            thread::sleep(Duration::from_millis(50));
            break;
        }
    }
    // Drain a bit more.
    for _ in 0..20 {
        executor
            .spin_once(Some(Duration::from_millis(20)))
            .expect("spin");
    }
    assert_eq!(
        max_seen.load(Ordering::SeqCst),
        1,
        "mutually exclusive group must not overlap"
    );
}

#[test]
fn reentrant_group_allows_parallel_timers() {
    let barrier = Arc::new(Barrier::new(2));
    let entered = Arc::new(Mutex::new(0usize));
    let parallel = Arc::new(AtomicUsize::new(0));

    let mut node = Node::new("cg");
    let executor = MultiThreadedExecutor::new(4);
    executor.add_node(&mut node).expect("add_node");
    let group = node.create_callback_group(CallbackGroupType::Reentrant);

    for _ in 0..2 {
        let barrier = barrier.clone();
        let entered = entered.clone();
        let parallel = parallel.clone();
        node.create_timer(
            Duration::from_millis(20),
            Arc::new(move || {
                {
                    let mut n = entered.lock().unwrap();
                    *n += 1;
                }
                barrier.wait();
                parallel.fetch_add(1, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(30));
            }),
            Some(&group),
        )
        .expect("timer");
    }

    let deadline = Instant::now() + Duration::from_secs(3);
    while parallel.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
        let _ = executor.spin_once(Some(Duration::from_millis(10)));
    }
    assert!(
        parallel.load(Ordering::SeqCst) >= 2,
        "reentrant timers should both pass the barrier (ran in parallel)"
    );
}

#[test]
fn spin_once_does_not_block_on_slow_worker_callbacks() {
    let mut node = Node::new("cg");
    let executor = MultiThreadedExecutor::new(2);
    executor.add_node(&mut node).expect("add_node");
    let group = node.create_callback_group(CallbackGroupType::Reentrant);

    for _ in 0..4 {
        node.create_timer(
            Duration::from_millis(1),
            Arc::new(|| thread::sleep(Duration::from_millis(80))),
            Some(&group),
        )
        .expect("timer");
    }

    thread::sleep(Duration::from_millis(5));
    let start = Instant::now();
    executor
        .spin_once(Some(Duration::from_millis(10)))
        .expect("spin");
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(100),
        "poll thread blocked on callbacks: {elapsed:?}"
    );
}

#[test]
fn distinct_mutually_exclusive_groups_run_in_parallel() {
    let barrier = Arc::new(Barrier::new(2));
    let parallel = Arc::new(AtomicUsize::new(0));

    let mut node = Node::new("cg");
    let executor = MultiThreadedExecutor::new(4);
    executor.add_node(&mut node).expect("add_node");

    for _ in 0..2 {
        let group = node.create_callback_group(CallbackGroupType::MutuallyExclusive);
        let barrier = barrier.clone();
        let parallel = parallel.clone();
        node.create_timer(
            Duration::from_millis(20),
            Arc::new(move || {
                barrier.wait();
                parallel.fetch_add(1, Ordering::SeqCst);
            }),
            Some(&group),
        )
        .expect("timer");
    }

    let deadline = Instant::now() + Duration::from_secs(3);
    while parallel.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
        let _ = executor.spin_once(Some(Duration::from_millis(10)));
    }
    assert!(
        parallel.load(Ordering::SeqCst) >= 2,
        "two mutually exclusive groups should run in parallel"
    );
}

#[test]
fn mutually_exclusive_group_queues_instead_of_occupying_every_worker() {
    let overlapping = Arc::new(AtomicUsize::new(0));
    let max_seen = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(AtomicUsize::new(0));

    let mut node = Node::new("cg");
    let executor = MultiThreadedExecutor::new(4);
    executor.add_node(&mut node).expect("add_node");
    let group = node.create_callback_group(CallbackGroupType::MutuallyExclusive);

    for _ in 0..4 {
        let overlapping = overlapping.clone();
        let max_seen = max_seen.clone();
        let started = started.clone();
        node.create_timer(
            Duration::from_millis(10),
            Arc::new(move || {
                started.fetch_add(1, Ordering::SeqCst);
                let now = overlapping.fetch_add(1, Ordering::SeqCst) + 1;
                max_seen.fetch_max(now, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(30));
                overlapping.fetch_sub(1, Ordering::SeqCst);
            }),
            Some(&group),
        )
        .expect("timer");
    }

    let deadline = Instant::now() + Duration::from_secs(3);
    while started.load(Ordering::SeqCst) < 4 && Instant::now() < deadline {
        let _ = executor.spin_once(Some(Duration::from_millis(10)));
    }
    thread::sleep(Duration::from_millis(150));
    assert_eq!(
        max_seen.load(Ordering::SeqCst),
        1,
        "queued mutually exclusive jobs must still run one at a time"
    );
}

#[test]
fn multi_threaded_callbacks_run_off_the_poll_thread() {
    let poll_id = thread::current().id();
    let cb_id = Arc::new(Mutex::new(None));

    let mut node = Node::new("cg");
    let executor = MultiThreadedExecutor::new(2);
    executor.add_node(&mut node).expect("add_node");
    let group = node.create_callback_group(CallbackGroupType::Reentrant);
    {
        let cb_id = cb_id.clone();
        node.create_timer(
            Duration::from_millis(5),
            Arc::new(move || {
                *cb_id.lock().unwrap() = Some(thread::current().id());
            }),
            Some(&group),
        )
        .expect("timer");
    }

    let deadline = Instant::now() + Duration::from_secs(2);
    while cb_id.lock().unwrap().is_none() && Instant::now() < deadline {
        let _ = executor.spin_once(Some(Duration::from_millis(10)));
    }
    let cb_id = cb_id.lock().unwrap().expect("callback should have run");
    assert_ne!(
        cb_id, poll_id,
        "callback must run on a worker thread, not the poll thread"
    );
}
