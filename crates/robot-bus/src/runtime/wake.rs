//! Interrupts the executor's `zmq::poll` when another thread hands it work.
//!
//! Replies and commands queued on process-local channels need a readable poll
//! item to end the wait. The Linux executor uses an eventfd with an inproc PAIR
//! as the portable fallback.

use crate::zmq;
use std::io;
#[cfg(target_os = "linux")]
use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SendError, Sender};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

struct Shared {
    /// Set while a notification is in flight, so bursts send at most one.
    pending: AtomicBool,
    signal: Signal,
}

enum Signal {
    Pair(Mutex<zmq::Socket>),
    #[cfg(target_os = "linux")]
    Event(Arc<OwnedFd>),
}

impl Signal {
    fn send(&self) -> io::Result<()> {
        match self {
            Self::Pair(socket) => socket
                .lock()
                .map_err(|_| io::Error::other("poisoned wake socket"))?
                .send(&[], zmq::DONTWAIT)
                .map_err(io::Error::other),
            #[cfg(target_os = "linux")]
            Self::Event(fd) => {
                let increment = 1u64.to_ne_bytes();
                // SAFETY: fd keeps the eventfd open and write reads eight bytes.
                eventfd_io(|| unsafe {
                    libc::write(fd.as_raw_fd(), increment.as_ptr().cast(), increment.len())
                })
            }
        }
    }
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
        if let Err(err) = shared.signal.send() {
            shared.pending.store(false, Ordering::Release);
            log::debug!("executor wake send failed: {err}");
        }
    }
}

/// Poll-thread end of a [`Waker`].
pub(crate) struct WakeReceiver {
    socket: ReceiverSocket,
    shared: Arc<Shared>,
}

enum ReceiverSocket {
    Pair(zmq::Socket),
    #[cfg(target_os = "linux")]
    Event(Arc<OwnedFd>),
}

impl WakeReceiver {
    pub(crate) fn poll_item(&self) -> zmq::PollItem<'_> {
        match &self.socket {
            ReceiverSocket::Pair(socket) => socket.as_poll_item(zmq::POLLIN),
            #[cfg(target_os = "linux")]
            ReceiverSocket::Event(fd) => zmq::PollItem::readable_fd(fd.as_fd()),
        }
    }

    /// Consume wake-ups. Must run before draining the queues they announce:
    /// anything enqueued after this either is drained now or wakes again.
    pub(crate) fn clear(&self) {
        match &self.socket {
            ReceiverSocket::Pair(socket) => while socket.recv_bytes(zmq::DONTWAIT).is_ok() {},
            #[cfg(target_os = "linux")]
            ReceiverSocket::Event(fd) => {
                let mut count = [0u8; 8];
                // SAFETY: fd keeps the eventfd open and count is a writable eight-byte buffer.
                if let Err(err) = eventfd_io(|| unsafe {
                    libc::read(fd.as_raw_fd(), count.as_mut_ptr().cast(), count.len())
                }) {
                    log::debug!("executor wake read failed: {err}");
                }
            }
        }
        self.shared.pending.swap(false, Ordering::AcqRel);
    }
}

#[cfg(target_os = "linux")]
fn eventfd_io(mut call: impl FnMut() -> isize) -> io::Result<()> {
    loop {
        if call() >= 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        match error.kind() {
            io::ErrorKind::Interrupted => continue,
            // A full counter is already readable. An empty one is already drained.
            io::ErrorKind::WouldBlock => return Ok(()),
            _ => return Err(error),
        }
    }
}

pub(crate) fn executor_wake_pair(context: &zmq::Context) -> (Waker, Option<WakeReceiver>) {
    #[cfg(target_os = "linux")]
    match eventfd_wake_pair() {
        Ok((waker, receiver)) => return (waker, Some(receiver)),
        Err(err) => log::debug!("executor eventfd unavailable: {err}"),
    }
    wake_pair(context)
}

#[cfg(target_os = "linux")]
fn eventfd_wake_pair() -> io::Result<(Waker, WakeReceiver)> {
    // SAFETY: eventfd takes a counter value and flags, and returns an owned descriptor.
    let fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the successful call created a descriptor with no other owner.
    let fd = Arc::new(unsafe { OwnedFd::from_raw_fd(fd) });
    Ok(wake_endpoints(
        Signal::Event(Arc::clone(&fd)),
        ReceiverSocket::Event(fd),
    ))
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
    Ok(wake_endpoints(
        Signal::Pair(Mutex::new(sender)),
        ReceiverSocket::Pair(receiver),
    ))
}

fn wake_endpoints(signal: Signal, socket: ReceiverSocket) -> (Waker, WakeReceiver) {
    let shared = Arc::new(Shared {
        pending: AtomicBool::new(false),
        signal,
    });
    (
        Waker {
            shared: Some(Arc::clone(&shared)),
        },
        WakeReceiver { socket, shared },
    )
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
        match &receiver.socket {
            ReceiverSocket::Pair(socket) => {
                socket.recv_bytes(zmq::DONTWAIT).unwrap();
                assert!(socket.recv_bytes(zmq::DONTWAIT).is_err());
            }
            #[cfg(target_os = "linux")]
            ReceiverSocket::Event(_) => panic!("expected a PAIR wake socket"),
        }
        receiver.clear();
        waker.wake();
        assert!(readable(&receiver, 1000));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn eventfd_wake_drains_and_rearms() {
        let (waker, receiver) = eventfd_wake_pair().unwrap();
        assert!(!readable(&receiver, 0));
        for _ in 0..100 {
            waker.wake();
        }
        assert!(readable(&receiver, 0));
        receiver.clear();
        assert!(!readable(&receiver, 0));
        std::thread::spawn(move || waker.wake()).join().unwrap();
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
