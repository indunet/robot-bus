//! Static action-bus peer endpoints for broker↔broker action federation.

use anyhow::{Context, Result, bail};

use super::ports::{BACKEND_PORT, FRONTEND_PORT};

/// Remote broker action-bus backend (federation DEALERs connect here).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionPeer {
    /// Peer action backend — local federation DEALER connects here.
    pub backend: String,
    /// Peer broker id for hop-path / no-advertise-back. Empty → learned later if possible.
    pub broker_id: String,
}

impl ActionPeer {
    /// Build a peer from its backend endpoint.
    ///
    /// Accepts:
    /// - `tcp://host:port`, `host:port`, or bare `host` (port [`BACKEND_PORT`])
    /// - `broker-id=tcp://host:port` to set [`Self::broker_id`]
    pub fn from_backend(backend: &str) -> Result<Self> {
        let (broker_id, backend) = split_peer_id(backend);
        Ok(Self {
            backend: normalize_tcp_endpoint(backend, BACKEND_PORT),
            broker_id,
        })
    }

    /// Build a peer from its frontend endpoint; backend uses port + 1.
    pub fn from_frontend(frontend: &str) -> Result<Self> {
        let (broker_id, frontend) = split_peer_id(frontend);
        let frontend = normalize_tcp_endpoint(frontend, FRONTEND_PORT);
        let backend = backend_from_frontend(&frontend)?;
        Ok(Self { backend, broker_id })
    }
}

fn split_peer_id(s: &str) -> (String, &str) {
    if let Some((id, rest)) = s.split_once('=') {
        if !id.is_empty()
            && !id.contains("://")
            && (rest.contains("://") || rest.contains(':') || !rest.is_empty())
        {
            return (id.to_string(), rest);
        }
    }
    (String::new(), s)
}

fn normalize_tcp_endpoint(addr: &str, default_port: u16) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else if addr.contains(':') {
        format!("tcp://{addr}")
    } else {
        format!("tcp://{addr}:{default_port}")
    }
}

fn backend_from_frontend(frontend: &str) -> Result<String> {
    let rest = frontend
        .strip_prefix("tcp://")
        .with_context(|| format!("action peer must be tcp://…, got {frontend}"))?;
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
            .context("peer frontend endpoint missing port")?;
        (host.to_string(), port_str)
    };
    let port: u16 = port_str
        .parse()
        .with_context(|| format!("invalid peer frontend port: {port_str}"))?;
    if port == u16::MAX {
        bail!("peer frontend port too high to derive backend (port + 1)");
    }
    Ok(format!("tcp://{host}:{}", port + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_backend_explicit() {
        let p = ActionPeer::from_backend("tcp://127.0.0.1:16665").unwrap();
        assert_eq!(p.backend, "tcp://127.0.0.1:16665");
        assert!(p.broker_id.is_empty());
    }

    #[test]
    fn from_backend_with_id() {
        let p = ActionPeer::from_backend("broker-b=tcp://127.0.0.1:16665").unwrap();
        assert_eq!(p.backend, "tcp://127.0.0.1:16665");
        assert_eq!(p.broker_id, "broker-b");
    }

    #[test]
    fn from_frontend_derives_backend_plus_one() {
        let p = ActionPeer::from_frontend("tcp://127.0.0.1:16664").unwrap();
        assert_eq!(p.backend, "tcp://127.0.0.1:16665");
    }

    #[test]
    fn bare_host_uses_default_backend_port() {
        let p = ActionPeer::from_backend("10.0.0.2").unwrap();
        assert_eq!(p.backend, format!("tcp://10.0.0.2:{BACKEND_PORT}"));
    }

    #[test]
    fn ipv6_from_frontend() {
        let p = ActionPeer::from_frontend("tcp://[::1]:15664").unwrap();
        assert_eq!(p.backend, "tcp://[::1]:15665");
    }
}
