//! Client-side discovery wait.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::time::{Duration, Instant};

use super::config::DiscoverOpts;
use super::net::{join_multicast_receiver, set_read_timeout};
use super::packet::{BrokerAnnouncement, try_parse_datagram};
use crate::errors::{BusError, Result};

/// Listen for UDP announces until `timeout`, then select one matching broker.
///
/// Invalid datagrams are discarded. When multiple brokers match the domain and
/// no `broker_id` filter is set, returns an error listing candidate ids.
pub fn wait(opts: DiscoverOpts) -> Result<BrokerAnnouncement> {
    let sock = join_multicast_receiver(opts.multicast_addr, opts.multicast_port)
        .map_err(|e| BusError::Protocol(format!("discovery join multicast: {e}")))?;

    let deadline = Instant::now() + opts.timeout;
    let mut by_id: HashMap<String, BrokerAnnouncement> = HashMap::new();
    let mut buf = [0u8; 2048];

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        let slice = remaining.min(Duration::from_millis(200));
        set_read_timeout(&sock, Some(slice))
            .map_err(|e| BusError::Protocol(format!("discovery set_read_timeout: {e}")))?;

        match sock.recv_from(&mut buf) {
            Ok((n, _)) => {
                let Some(ann) = try_parse_datagram(&buf[..n]) else {
                    continue;
                };
                if ann.domain_id != opts.domain_id {
                    continue;
                }
                if let Some(want) = opts.broker_id.as_deref() {
                    if ann.broker_id != want {
                        continue;
                    }
                    return Ok(ann);
                }
                by_id.insert(ann.broker_id.clone(), ann);
            }
            Err(e) if is_timeout(&e) => continue,
            Err(e) => {
                return Err(BusError::Protocol(format!("discovery recv: {e}")));
            }
        }
    }

    if let Some(want) = opts.broker_id.as_deref() {
        return Err(BusError::Protocol(format!(
            "discovery timed out waiting for broker_id={want:?} domain_id={}",
            opts.domain_id
        )));
    }

    match by_id.len() {
        0 => Err(BusError::Protocol(format!(
            "discovery timed out (no broker on domain_id={})",
            opts.domain_id
        ))),
        1 => Ok(by_id.into_values().next().expect("len 1")),
        _ => {
            let mut ids: Vec<_> = by_id.keys().cloned().collect();
            ids.sort();
            Err(BusError::Protocol(format!(
                "discovery found multiple brokers on domain_id={}: {}; specify broker_id",
                opts.domain_id,
                ids.join(", ")
            )))
        }
    }
}

fn is_timeout(err: &std::io::Error) -> bool {
    matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}
