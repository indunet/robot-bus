//! Interrupts the executor's `zmq::poll` when another thread hands it work.
//!
//! The poll set only watches ZeroMQ sockets, so replies and commands queued on
//! process-local channels need a readable socket to end the wait. An inproc
//! PAIR provides one on every platform libzmq supports.

use crate::zmq;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SendError, Sender};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

struct Shared {
    /// Set while a wake message is in flight, so bursts send at most one.
    pending: AtomicBool,
    socket: Mutex<zmq::Socket>,
}

/// Cloneable, thread-safe handle that wakes the poll thread.
#[derive(Clone)]
pub(crate) struct Waker {
    shared: Option<Arc<Shared>>,
}

impl Waker {
    /// Never wakes; the poll thread falls back to its timeout.
    pub(crate) fn disabled() -> Self {
        Self { shared: None }
    }

    pub(crate) fn wake(&self) {
        let Some(shared) = &self.shared else {
            return;
        };
        if shared.pending.swap(true, Ordering::AcqRel) {
            return;
        }
        let sent = match shared.socket.lock() {
            Ok(socket) => socket.send(&[][..], zmq::DONTWAIT),
            Err(_) => Err(zmq::Error::EFAULT),
        };
        if let Err(err) = sent {
            shared.pending.store(false, Ordering::Release);
            log::debug!("executor wake send failed: {err}");
        }
    }
}

/// Poll-thread end of a [`Waker`].
pub(crate) struct WakeReceiver {
    socket: zmq::Socket,
    shared: Arc<Shared>,
}

impl WakeReceiver {
    pub(crate) fn poll_item(&self) -> zmq::PollItem<'_> {
        self.socket.as_poll_item(zmq::POLLIN)
    }

    /// Consume wake-ups. Must run before draining the queues they announce:
    /// anything enqueued after this either is drained now or wakes again.
    pub(crate) fn clear(&self) {
        while self.socket.recv_bytes(zmq::DONTWAIT).is_ok() {}
        self.shared.pending.swap(false, Ordering::AcqRel);
    }
}

/// Create a connected waker pair, or a disabled waker if sockets are unavailable.
pub(crate) fn wake_pair(context: &zmq::Context) -> (Waker, Option<WakeReceiver>) {
    match try_wake_pair(context) {
        Ok((waker, receiver)) => (waker, Some(receiver)),
        Err(err) => {
            log::warn!("executor wake socket unavailable, replies wait for poll timeout: {err}");
            (Waker::disabled(), None)
        }
    }
}

fn try_wake_pair(context: &zmq::Context) -> Result<(Waker, WakeReceiver), zmq::Error> {
    let endpoint = format!(
        "inproc://robot-bus-executor-wake-{}",
        Uuid::new_v4().simple()
    );
    let receiver = context.socket(zmq::SocketType::PAIR)?;
    let sender = context.socket(zmq::SocketType::PAIR)?;
    receiver.set_linger(0)?;
    sender.set_linger(0)?;
    receiver.bind(&endpoint)?;
    sender.connect(&endpoint)?;
    let shared = Arc::new(Shared {
        pending: AtomicBool::new(false),
        socket: Mutex::new(sender),
    });
    Ok((
        Waker {
            shared: Some(Arc::clone(&shared)),
        },
        WakeReceiver {
            socket: receiver,
            shared,
        },
    ))
}

/// Channel sender that wakes the poll thread after each enqueue.
pub(crate) struct WakingSender<T> {
    tx: Sender<T>,
    waker: Waker,
}

impl<T> Clone for WakingSender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            waker: self.waker.clone(),
        }
    }
}

impl<T> WakingSender<T> {
    pub(crate) fn new(tx: Sender<T>, waker: Waker) -> Self {
        Self { tx, waker }
    }

    pub(crate) fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.tx.send(value)?;
        self.waker.wake();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn readable(receiver: &WakeReceiver, timeout_ms: i64) -> bool {
        let mut items = [receiver.poll_item()];
        zmq::poll(&mut items, timeout_ms).unwrap() > 0 && items[0].is_readable()
    }

    #[test]
    fn wake_before_poll_is_kept_until_cleared() {
        let context = zmq::Context::new();
        let (waker, receiver) = wake_pair(&context);
        let receiver = receiver.unwrap();
        assert!(!readable(&receiver, 0));
        waker.wake();
        assert!(readable(&receiver, 1000));
        receiver.clear();
        assert!(!readable(&receiver, 0));
    }

    #[test]
    fn burst_of_wakes_sends_one_message_until_cleared() {
        let context = zmq::Context::new();
        let (waker, receiver) = wake_pair(&context);
        let receiver = receiver.unwrap();
        for _ in 0..100 {
            waker.clone().wake();
        }
        assert!(readable(&receiver, 1000));
        receiver.socket.recv_bytes(zmq::DONTWAIT).unwrap();
        assert!(receiver.socket.recv_bytes(zmq::DONTWAIT).is_err());
        receiver.clear();
        waker.wake();
        assert!(readable(&receiver, 1000));
    }

    #[test]
    fn waking_sender_wakes_from_another_thread() {
        let context = zmq::Context::new();
        let (waker, receiver) = wake_pair(&context);
        let receiver = receiver.unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let sender = WakingSender::new(tx, waker);
        std::thread::spawn(move || sender.send(7).unwrap());
        assert!(readable(&receiver, 2000));
        receiver.clear();
        assert_eq!(rx.try_recv().unwrap(), 7);
    }
}
