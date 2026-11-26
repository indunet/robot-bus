//! Static message-bus peer endpoints for broker↔broker topic federation.

use anyhow::{Context, Result, bail};

use crate::transports::{XPUB_CHANNEL, XPUB_PORT, XSUB_CHANNEL, ipc_sibling_endpoint};

/// Remote broker message-bus endpoints (client-facing XSUB/XPUB of the peer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessagePeer {
    /// Peer XPUB — local bridge SUB connects here to express topic demand.
    pub xpub: String,
    /// Peer XSUB — local bridge PUB connects here to push federated frames.
    pub xsub: String,
    /// Peer broker id. Empty → unknown (the link is then always active; see
    /// [`link ownership`](super)).
    pub broker_id: String,
    /// Peer federation channel (ROUTER). `Some` → the peer supports one-sided join and
    /// the link uses it; `None` → legacy XPUB/XSUB links (both sides must configure peers).
    pub fed: Option<String>,
}

impl MessagePeer {
    /// Build a peer from its XPUB endpoint; XSUB uses the same host with port − 1.
    ///
    /// Accepts `tcp://host:port`, `host:port`, or a bare `host`
    /// (XPUB [`XPUB_PORT`] / XSUB = XPUB − 1), or the peer's
    /// `ipc://DIR/message_bus_xpub.ipc` (XSUB = `DIR/message_bus_xsub.ipc`).
    pub fn from_xpub(xpub: &str) -> Result<Self> {
        let xpub = normalize_tcp_endpoint(xpub);
        let xsub = xsub_from_xpub(&xpub)?;
        Ok(Self {
            xpub,
            xsub,
            broker_id: String::new(),
            fed: None,
        })
    }
}

fn normalize_tcp_endpoint(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else if addr.contains(':') {
        format!("tcp://{addr}")
    } else {
        format!("tcp://{addr}:{XPUB_PORT}")
    }
}

fn xsub_from_xpub(xpub: &str) -> Result<String> {
    if xpub.starts_with("ipc://") {
        return ipc_sibling_endpoint(xpub, XPUB_CHANNEL, XSUB_CHANNEL).with_context(|| {
            format!("message peer ipc endpoint must end with /message_bus_xpub.ipc, got {xpub}")
        });
    }
    let rest = xpub
        .strip_prefix("tcp://")
        .with_context(|| format!("message peer must be tcp://… or ipc://…, got {xpub}"))?;
    // IPv6: tcp://[::1]:15581
    let (host, port_str) = if let Some(bracket) = rest.strip_prefix('[') {
        let end = bracket
            .find("]:")
            .context("invalid IPv6 peer endpoint (expected tcp://[addr]:port)")?;
        let host = format!("[{}]", &bracket[..end]);
        let port_str = &bracket[end + 2..];
        (host, port_str)
    } else {
        let (host, port_str) = rest
            .rsplit_once(':')
            .context("peer XPUB endpoint missing port")?;
        (host.to_string(), port_str)
    };
    let port: u16 = port_str
        .parse()
        .with_context(|| format!("invalid peer XPUB port: {port_str}"))?;
    if port == 0 {
        bail!("peer XPUB port must be > 0 so XSUB can use port - 1");
    }
    Ok(format!("tcp://{host}:{}", port - 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transports::XSUB_PORT;

    #[test]
    fn derives_xsub_port_minus_one() {
        let p = MessagePeer::from_xpub("tcp://127.0.0.1:16561").unwrap();
        assert_eq!(p.xpub, "tcp://127.0.0.1:16561");
        assert_eq!(p.xsub, "tcp://127.0.0.1:16560");
    }

    #[test]
    fn accepts_host_port_without_scheme() {
        let p = MessagePeer::from_xpub(&format!("10.0.0.2:{XPUB_PORT}")).unwrap();
        assert_eq!(p.xpub, format!("tcp://10.0.0.2:{XPUB_PORT}"));
        assert_eq!(p.xsub, format!("tcp://10.0.0.2:{XSUB_PORT}"));
    }

    #[test]
    fn ipv6_peer() {
        let p = MessagePeer::from_xpub(&format!("tcp://[::1]:{XPUB_PORT}")).unwrap();
        assert_eq!(p.xpub, format!("tcp://[::1]:{XPUB_PORT}"));
        assert_eq!(p.xsub, format!("tcp://[::1]:{XSUB_PORT}"));
    }

    #[test]
    fn ipc_peer_derives_xsub_file() {
        let p = MessagePeer::from_xpub("ipc:///tmp/rb-a/message_bus_xpub.ipc").unwrap();
        assert_eq!(p.xpub, "ipc:///tmp/rb-a/message_bus_xpub.ipc");
        assert_eq!(p.xsub, "ipc:///tmp/rb-a/message_bus_xsub.ipc");
    }

    #[test]
    fn ipc_peer_rejects_non_xpub_file() {
        assert!(MessagePeer::from_xpub("ipc:///tmp/rb-a/message_bus_xsub.ipc").is_err());
    }
}
