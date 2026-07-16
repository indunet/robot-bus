//! Callback group concurrency smoke tests.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use robot_bus::{
    CallbackGroupType, MultiThreadedExecutor, Node,
};

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
