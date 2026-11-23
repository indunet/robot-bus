//! Shared ZeroMQ socket helpers.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zmq::Socket;

/// ZeroMQ send / receive high-water marks (max queued messages per direction).
///
/// Topic [`crate::QosProfile::keep_last`] depth maps here. When the queue is full,
/// further sends block or drop depending on socket type / flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighWaterMark {
    /// `ZMQ_SNDHWM` — outbound queue depth.
    pub snd: i32,
    /// `ZMQ_RCVHWM` — inbound queue depth.
    pub rcv: i32,
}

impl HighWaterMark {
    /// Message bus (PUB/SUB) defaults — short queues for low latency, depth 8 for bursts.
    pub const STREAM: Self = Self { snd: 8, rcv: 8 };
    /// Service bus (REQ/DEALER) defaults.
    pub const RPC: Self = Self { snd: 4, rcv: 4 };
    /// Action bus defaults.
    pub const ACTION: Self = Self { snd: 8, rcv: 8 };

    pub const fn new(snd: i32, rcv: i32) -> Self {
        Self { snd, rcv }
    }

    pub fn apply(self, socket: &Socket) -> Result<(), zmq::Error> {
        socket.set_sndhwm(self.snd)?;
        socket.set_rcvhwm(self.rcv)?;
        Ok(())
    }

    pub fn from_socket(socket: &Socket) -> Result<Self, zmq::Error> {
        Ok(Self {
            snd: socket.get_sndhwm()?,
            rcv: socket.get_rcvhwm()?,
        })
    }
}

pub const STREAM_SND_HWM: i32 = HighWaterMark::STREAM.snd;
pub const STREAM_RCV_HWM: i32 = HighWaterMark::STREAM.rcv;
pub const RPC_SND_HWM: i32 = HighWaterMark::RPC.snd;
pub const RPC_RCV_HWM: i32 = HighWaterMark::RPC.rcv;
pub const ACTION_SND_HWM: i32 = HighWaterMark::ACTION.snd;
pub const ACTION_RCV_HWM: i32 = HighWaterMark::ACTION.rcv;

pub fn apply_publisher_options(socket: &Socket) -> Result<(), zmq::Error> {
    apply_publisher_options_with(socket, HighWaterMark::STREAM)
}

pub fn apply_publisher_options_with(socket: &Socket, hwm: HighWaterMark) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    hwm.apply(socket)?;
    socket.set_immediate(true)?;
    Ok(())
}

/// SUB must not use IMMEDIATE — it breaks XPUB subscription delivery.
pub fn apply_subscriber_options(socket: &Socket) -> Result<(), zmq::Error> {
    apply_subscriber_options_with(socket, HighWaterMark::STREAM)
}

pub fn apply_subscriber_options_with(
    socket: &Socket,
    hwm: HighWaterMark,
) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    hwm.apply(socket)?;
    Ok(())
}

pub fn apply_rpc_options(socket: &Socket) -> Result<(), zmq::Error> {
    apply_rpc_options_with(socket, HighWaterMark::RPC)
}

pub fn apply_rpc_options_with(socket: &Socket, hwm: HighWaterMark) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    hwm.apply(socket)?;
    socket.set_immediate(true)?;
    Ok(())
}

pub fn apply_action_options(socket: &Socket) -> Result<(), zmq::Error> {
    apply_action_options_with(socket, HighWaterMark::ACTION)
}

pub fn apply_action_options_with(socket: &Socket, hwm: HighWaterMark) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    hwm.apply(socket)?;
    socket.set_immediate(true)?;
    Ok(())
}

pub fn wait_for_connection() {
    thread::sleep(Duration::from_millis(50));
}

pub fn poll_readable(socket: &Socket, timeout_ms: i64) -> Result<bool, zmq::Error> {
    let mut items = [socket.as_poll_item(zmq::POLLIN)];
    Ok(zmq::poll(&mut items, timeout_ms)? > 0 && items[0].is_readable())
}

pub struct HeartbeatThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl HeartbeatThread {
    pub fn start<F>(send_heartbeat: F, interval: Duration) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let handle = thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                thread::sleep(interval);
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                send_heartbeat();
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for HeartbeatThread {
    fn drop(&mut self) {
        self.stop();
    }
}
