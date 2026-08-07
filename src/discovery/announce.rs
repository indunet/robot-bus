//! Periodic broker announce sender.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::{debug, warn};

use super::config::DiscoveryConfig;
use super::net::{infer_advertise_host, multicast_sender};
use super::packet::{BrokerAnnouncement, encode_announce};
use crate::errors::Result;

/// Payload assembled once at broker start (ports / paths already resolved).
#[derive(Clone, Debug)]
pub struct AnnouncerPayload {
    pub announcement: BrokerAnnouncement,
}

/// Background announce loop handle.
pub struct AnnounceHandle {
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl AnnounceHandle {
    pub fn stop(mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for AnnounceHandle {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Spawn a background thread that sends `payload` every `config.interval`.
pub fn spawn_announcer(
    config: DiscoveryConfig,
    payload: AnnouncerPayload,
) -> Result<AnnounceHandle> {
    let (sock, dest) = multicast_sender(config.multicast_addr, config.multicast_port)
        .map_err(|e| crate::errors::BusError::Protocol(format!("discovery sender: {e}")))?;
    let bytes = encode_announce(&payload.announcement)?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = shutdown.clone();
    let interval = config.interval;
    let handle = thread::Builder::new()
        .name("robot-bus-discovery".into())
        .spawn(move || {
            while !flag.load(Ordering::Acquire) {
                if let Err(e) = sock.send_to(&bytes, dest) {
                    warn!("discovery announce send failed: {e}");
                } else {
                    debug!(
                        "discovery announce sent to {}:{} domain={}",
                        config.multicast_addr,
                        config.multicast_port,
                        payload.announcement.domain_id
                    );
                }
                // Short sleeps so shutdown is responsive.
                let slice = Duration::from_millis(50);
                let mut waited = Duration::ZERO;
                while waited < interval && !flag.load(Ordering::Acquire) {
                    thread::sleep(slice);
                    waited += slice;
                }
            }
        })
        .map_err(|e| crate::errors::BusError::Protocol(format!("spawn discovery: {e}")))?;
    Ok(AnnounceHandle {
        shutdown,
        handle: Some(handle),
    })
}

/// Resolve advertise host from config or inference.
pub fn resolve_advertise_host(config: &DiscoveryConfig) -> String {
    config
        .advertise_host
        .clone()
        .filter(|s| !s.is_empty() && s != "0.0.0.0" && s != "*")
        .unwrap_or_else(infer_advertise_host)
}
