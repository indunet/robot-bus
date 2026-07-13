//! Shared ZeroMQ socket helpers.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zmq::Socket;

pub const STREAM_SND_HWM: i32 = 2;
pub const STREAM_RCV_HWM: i32 = 2;
pub const RPC_SND_HWM: i32 = 4;
pub const RPC_RCV_HWM: i32 = 4;
pub const ACTION_SND_HWM: i32 = 8;
pub const ACTION_RCV_HWM: i32 = 8;

pub fn apply_publisher_options(socket: &Socket) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    socket.set_sndhwm(STREAM_SND_HWM)?;
    socket.set_rcvhwm(STREAM_RCV_HWM)?;
    socket.set_immediate(true)?;
    Ok(())
}

/// SUB must not use IMMEDIATE — it breaks XPUB subscription delivery.
pub fn apply_subscriber_options(socket: &Socket) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    socket.set_sndhwm(STREAM_SND_HWM)?;
    socket.set_rcvhwm(STREAM_RCV_HWM)?;
    Ok(())
}

pub fn apply_rpc_options(socket: &Socket) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    socket.set_sndhwm(RPC_SND_HWM)?;
    socket.set_rcvhwm(RPC_RCV_HWM)?;
    socket.set_immediate(true)?;
    Ok(())
}

pub fn apply_action_options(socket: &Socket) -> Result<(), zmq::Error> {
    socket.set_linger(0)?;
    socket.set_sndhwm(ACTION_SND_HWM)?;
    socket.set_rcvhwm(ACTION_RCV_HWM)?;
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
