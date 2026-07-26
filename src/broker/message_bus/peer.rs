//! Static message-bus peer endpoints for broker↔broker topic federation.

use anyhow::{bail, Context, Result};

/// Remote broker message-bus endpoints (client-facing XSUB/XPUB of the peer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessagePeer {
    /// Peer XPUB — local bridge SUB connects here to express topic demand.
    pub xpub: String,
    /// Peer XSUB — local bridge PUB connects here to push federated frames.
    pub xsub: String,
}

impl MessagePeer {
    /// Build a peer from its XPUB endpoint; XSUB uses the same host with port − 1.
    ///
    /// Accepts `tcp://host:port`, `host:port`, or a bare `host` (ports 15561 / 15560).
    pub fn from_xpub(xpub: &str) -> Result<Self> {
        let xpub = normalize_tcp_endpoint(xpub);
        let xsub = xsub_from_xpub(&xpub)?;
        Ok(Self { xpub, xsub })
    }
}

fn normalize_tcp_endpoint(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else if addr.contains(':') {
        format!("tcp://{addr}")
    } else {
        format!("tcp://{addr}:15561")
    }
}

fn xsub_from_xpub(xpub: &str) -> Result<String> {
    let rest = xpub
        .strip_prefix("tcp://")
        .with_context(|| format!("message peer must be tcp://…, got {xpub}"))?;
    // IPv6: tcp://[::1]:15561
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

    #[test]
    fn derives_xsub_port_minus_one() {
        let p = MessagePeer::from_xpub("tcp://127.0.0.1:16561").unwrap();
        assert_eq!(p.xpub, "tcp://127.0.0.1:16561");
        assert_eq!(p.xsub, "tcp://127.0.0.1:16560");
    }

    #[test]
    fn accepts_host_port_without_scheme() {
        let p = MessagePeer::from_xpub("10.0.0.2:15561").unwrap();
        assert_eq!(p.xpub, "tcp://10.0.0.2:15561");
        assert_eq!(p.xsub, "tcp://10.0.0.2:15560");
    }

    #[test]
    fn ipv6_peer() {
        let p = MessagePeer::from_xpub("tcp://[::1]:15561").unwrap();
        assert_eq!(p.xpub, "tcp://[::1]:15561");
        assert_eq!(p.xsub, "tcp://[::1]:15560");
    }
}
