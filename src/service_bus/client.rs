//! REQ client for the service bus frontend.
//!
//! The REQ socket is guarded by a [`Mutex`] so a single client handle is `Send +
//! Sync` and safe to call from multiple threads (calls are serialised; one
//! in-flight REQ at a time per handle).

use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};

use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use crate::errors::{BusError, Result, parse_error_body};
use crate::transports;
use crate::zmq_helpers::{HighWaterMark, apply_rpc_options_with};

pub struct ServiceClient {
    context: Context,
    endpoint: String,
    hwm: Mutex<HighWaterMark>,
    socket: Mutex<Socket>,
}

impl ServiceClient {
    pub fn new(endpoint: Option<&str>) -> Result<Self> {
        Self::with_hwm(endpoint, HighWaterMark::RPC)
    }

    pub fn with_hwm(endpoint: Option<&str>, hwm: HighWaterMark) -> Result<Self> {
        Self::with_context_hwm(&Context::new(), endpoint, hwm)
    }

    /// Create a client using a shared ZeroMQ context (required for inproc).
    pub fn with_context_hwm(
        context: &Context,
        endpoint: Option<&str>,
        hwm: HighWaterMark,
    ) -> Result<Self> {
        let endpoint = match endpoint {
            Some(ep) => ep.to_string(),
            None => transports::service_frontend_endpoint("localhost", "tcp")
                .map_err(|e| BusError::Protocol(e))?,
        };
        let socket = Self::connect_socket(context, &endpoint, hwm)?;
        log::info!("service client connected to {endpoint}");
        Ok(Self {
            context: context.clone(),
            endpoint,
            hwm: Mutex::new(hwm),
            socket: Mutex::new(socket),
        })
    }

    fn connect_socket(context: &Context, endpoint: &str, hwm: HighWaterMark) -> Result<Socket> {
        let socket = context.socket(SocketType::REQ)?;
        apply_rpc_options_with(&socket, hwm)?;
        socket.connect(endpoint)?;
        Ok(socket)
    }

    fn lock_socket(&self) -> Result<std::sync::MutexGuard<'_, Socket>> {
        self.socket
            .lock()
            .map_err(|_| BusError::Protocol("service client socket mutex poisoned".into()))
    }

    fn lock_hwm(&self) -> Result<std::sync::MutexGuard<'_, HighWaterMark>> {
        self.hwm
            .lock()
            .map_err(|_| BusError::Protocol("service client hwm mutex poisoned".into()))
    }

    /// Recreate the REQ socket after timeout / protocol errors leave it unusable.
    /// Caller must already hold `socket` (and ideally not hold other locks that
    /// could deadlock); this takes `hwm` then replaces `socket`.
    fn reset_socket_locked(&self, sock: &mut Socket) -> Result<()> {
        let hwm = *self.lock_hwm()?;
        let _ = sock.set_linger(0);
        *sock = Self::connect_socket(&self.context, &self.endpoint, hwm)?;
        Ok(())
    }

    fn lock_socket_until(
        &self,
        deadline: Option<Instant>,
        service: &str,
    ) -> Result<MutexGuard<'_, Socket>> {
        let Some(deadline) = deadline else {
            return self.lock_socket();
        };
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|duration| !duration.is_zero())
                .ok_or_else(|| timeout_error(service))?;
            match self.socket.try_lock() {
                Ok(socket) => return Ok(socket),
                Err(TryLockError::Poisoned(_)) => {
                    return Err(BusError::Protocol(
                        "service client socket mutex poisoned".into(),
                    ));
                }
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(remaining.min(Duration::from_millis(1)))
                }
            }
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn high_water_mark(&self) -> Result<HighWaterMark> {
        let sock = self.lock_socket()?;
        Ok(HighWaterMark::from_socket(&sock)?)
    }

    pub fn set_high_water_mark(&self, hwm: HighWaterMark) -> Result<()> {
        let sock = self.lock_socket()?;
        hwm.apply(&sock)?;
        *self.lock_hwm()? = hwm;
        Ok(())
    }

    pub fn call(
        &self,
        service_name: &str,
        body: &[u8],
        request_id: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        let deadline = timeout
            .map(|duration| {
                Instant::now()
                    .checked_add(duration)
                    .ok_or_else(|| BusError::Protocol("service timeout is too large".into()))
            })
            .transpose()?;
        self.call_with_deadline(service_name, body, request_id, deadline)
    }

    /// Share one deadline across server queuing, socket locking, send and receive.
    pub(crate) fn call_with_deadline(
        &self,
        service_name: &str,
        body: &[u8],
        request_id: Option<&str>,
        deadline: Option<Instant>,
    ) -> Result<Vec<u8>> {
        let req_id = request_id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
        // A concurrent call must not spend an unlimited time waiting for this lock.
        let mut sock = self.lock_socket_until(deadline, service_name)?;
        let exchange = (|| -> Result<Vec<Vec<u8>>> {
            // Always reset both timeouts, including finite -> unlimited reuse.
            sock.set_sndtimeo(remaining_ms(deadline, service_name)?)?;
            sock.send_multipart([service_name.as_bytes(), req_id.as_bytes(), body], 0)?;
            sock.set_rcvtimeo(remaining_ms(deadline, service_name)?)?;
            Ok(sock.recv_multipart(0)?)
        })();
        let frames = match exchange {
            Ok(frames) => frames,
            Err(err) => {
                // Send failures as well as receive failures can leave REQ mid-exchange.
                let _ = self.reset_socket_locked(&mut sock);
                return Err(match err {
                    BusError::Zmq(zmq::Error::EAGAIN) if deadline.is_some() => {
                        timeout_error(service_name)
                    }
                    other => other,
                });
            }
        };
        if frames.len() != 3 {
            let _ = self.reset_socket_locked(&mut sock);
            return Err(BusError::Protocol(format!(
                "expected 3 reply frames, got {}",
                frames.len()
            )));
        }
        let reply_svc = String::from_utf8_lossy(&frames[0]);
        let reply_id = String::from_utf8_lossy(&frames[1]);
        if reply_svc != service_name {
            let _ = self.reset_socket_locked(&mut sock);
            return Err(BusError::Protocol(format!(
                "service name mismatch: {reply_svc:?}"
            )));
        }
        if reply_id != req_id {
            let _ = self.reset_socket_locked(&mut sock);
            return Err(BusError::Protocol(format!(
                "request id mismatch: {reply_id:?}"
            )));
        }
        if let Some(err) = parse_error_body(&frames[2]) {
            return Err(err);
        }
        Ok(frames[2].clone())
    }
}

fn timeout_error(service: &str) -> BusError {
    BusError::Timeout(format!("service '{service}' timed out"))
}

fn remaining_ms(deadline: Option<Instant>, service: &str) -> Result<i32> {
    match deadline {
        None => Ok(-1),
        Some(deadline) => {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|duration| !duration.is_zero())
                .ok_or_else(|| timeout_error(service))?;
            Ok(remaining.as_millis().clamp(1, i32::MAX as u128) as i32)
        }
    }
}

impl Drop for ServiceClient {
    fn drop(&mut self) {
        if let Ok(sock) = self.socket.get_mut() {
            let _ = sock.set_linger(0);
        }
    }
}

#[cfg(test)]
mod sync_assert {
    use super::ServiceClient;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn service_client_is_send_sync() {
        assert_send_sync::<ServiceClient>();
    }
}

#[cfg(test)]
mod deadline_tests {
    use super::*;
    use std::sync::{Arc, mpsc};
    use std::thread;

    fn server() -> (Socket, String) {
        let context = Context::new();
        let server = context.socket(SocketType::REP).unwrap();
        server.set_linger(1000).unwrap();
        server.set_rcvtimeo(2000).unwrap();
        server.bind("tcp://127.0.0.1:*").unwrap();
        let endpoint = server.get_last_endpoint().unwrap().unwrap();
        (server, endpoint)
    }

    #[test]
    fn timeout_while_waiting_for_socket_does_not_send_or_reset_active_call() {
        let (server, endpoint) = server();
        let client = Arc::new(ServiceClient::new(Some(&endpoint)).unwrap());
        let (received_tx, received_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let frames = server.recv_multipart(0).unwrap();
            received_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            server.send_multipart(frames, 0).unwrap();
            server // keep the peer connected until the caller has received the reply
        });
        let first_client = Arc::clone(&client);
        let first = thread::spawn(move || {
            first_client.call("echo", b"first", None, Some(Duration::from_secs(2)))
        });
        received_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let start = Instant::now();
        let second = client.call("echo", b"second", None, Some(Duration::from_millis(40)));
        let elapsed = start.elapsed();
        release_tx.send(()).unwrap();
        let first_result = first.join().unwrap();
        worker.join().unwrap();
        assert!(matches!(second, Err(BusError::Timeout(_))));
        assert!(elapsed < Duration::from_millis(500));
        assert_eq!(first_result.unwrap(), b"first");
    }

    #[test]
    fn send_timeout_is_typed_and_socket_can_be_reused() {
        let context = Context::new();
        let reservation = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("tcp://{}", reservation.local_addr().unwrap());
        let client = ServiceClient::new(Some(&endpoint)).unwrap();
        let result = client.call("echo", b"expired", None, Some(Duration::from_millis(40)));
        assert!(matches!(result, Err(BusError::Timeout(_))), "{result:?}");
        drop(reservation);
        let server = context.socket(SocketType::REP).unwrap();
        server.set_linger(1000).unwrap();
        server.set_rcvtimeo(2000).unwrap();
        server.bind(&endpoint).unwrap();
        let worker = thread::spawn(move || {
            let frames = server.recv_multipart(0).unwrap();
            assert_eq!(frames[2], b"retry");
            server.send_multipart(frames, 0).unwrap();
            server // keep the peer connected until the caller has received the reply
        });
        let reply = client.call("echo", b"retry", None, Some(Duration::from_secs(2)));
        worker.join().unwrap();
        assert_eq!(reply.unwrap(), b"retry");
    }

    #[test]
    fn unlimited_call_resets_previous_socket_timeouts() {
        let (server, endpoint) = server();
        let client = ServiceClient::new(Some(&endpoint)).unwrap();
        let worker = thread::spawn(move || {
            for index in 0..2 {
                let frames = server.recv_multipart(0).unwrap();
                if index == 1 {
                    thread::sleep(Duration::from_millis(150));
                }
                server.send_multipart(frames, 0).unwrap();
            }
            server
        });
        client
            .call("echo", b"finite", None, Some(Duration::from_secs(1)))
            .unwrap();
        // Simulate short timeout values left over by a prior successful call.
        {
            let socket = client.lock_socket().unwrap();
            socket.set_sndtimeo(10).unwrap();
            socket.set_rcvtimeo(10).unwrap();
        }
        let reply = client.call("echo", b"unlimited", None, None);
        worker.join().unwrap();
        assert_eq!(reply.unwrap(), b"unlimited");
        let socket = client.lock_socket().unwrap();
        assert_eq!(socket.get_sndtimeo().unwrap(), -1);
        assert_eq!(socket.get_rcvtimeo().unwrap(), -1);
    }
}
