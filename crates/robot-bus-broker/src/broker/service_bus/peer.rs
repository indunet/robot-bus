//! Static service-bus peer endpoints for broker↔broker RPC federation.

use anyhow::{Context, Result, bail};

use robot_bus::transports::{
    SERVICE_BACKEND_CHANNEL as BACKEND_CHANNEL, SERVICE_BACKEND_PORT as BACKEND_PORT,
    SERVICE_FRONTEND_CHANNEL as FRONTEND_CHANNEL, SERVICE_FRONTEND_PORT as FRONTEND_PORT,
    ipc_sibling_endpoint,
};

/// Remote broker service-bus backend (workers / federation DEALERs connect here).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServicePeer {
    /// Peer service backend — local federation DEALER connects here.
    pub backend: String,
    /// Peer broker id for hop-path / no-advertise-back. Empty → learned later if possible.
    pub broker_id: String,
}

impl ServicePeer {
    /// Build a peer from its backend endpoint.
    ///
    /// Accepts:
    /// - `tcp://host:port`, `host:port`, or bare `host` (port [`BACKEND_PORT`])
    /// - `ipc://DIR/service_bus_backend.ipc` for a peer on the same host
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

/// `id=endpoint` → (id, endpoint); otherwise ("", input).
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
    if frontend.starts_with("ipc://") {
        return ipc_sibling_endpoint(frontend, FRONTEND_CHANNEL, BACKEND_CHANNEL).with_context(
            || {
                format!(
                    "service peer ipc endpoint must end with /service_bus_frontend.ipc, got {frontend}"
                )
            },
        );
    }
    let rest = frontend
        .strip_prefix("tcp://")
        .with_context(|| format!("service peer must be tcp://… or ipc://…, got {frontend}"))?;
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
        let p = ServicePeer::from_backend("tcp://127.0.0.1:16663").unwrap();
        assert_eq!(p.backend, "tcp://127.0.0.1:16663");
        assert!(p.broker_id.is_empty());
    }

    #[test]
    fn from_backend_with_id() {
        let p = ServicePeer::from_backend("broker-b=tcp://127.0.0.1:16663").unwrap();
        assert_eq!(p.backend, "tcp://127.0.0.1:16663");
        assert_eq!(p.broker_id, "broker-b");
    }

    #[test]
    fn from_frontend_derives_backend_plus_one() {
        let p = ServicePeer::from_frontend("tcp://127.0.0.1:16662").unwrap();
        assert_eq!(p.backend, "tcp://127.0.0.1:16663");
    }

    #[test]
    fn accepts_host_port_without_scheme() {
        let p = ServicePeer::from_backend("10.0.0.2:15663").unwrap();
        assert_eq!(p.backend, "tcp://10.0.0.2:15663");
    }

    #[test]
    fn bare_host_uses_default_backend_port() {
        let p = ServicePeer::from_backend("10.0.0.2").unwrap();
        assert_eq!(p.backend, format!("tcp://10.0.0.2:{BACKEND_PORT}"));
    }

    #[test]
    fn ipv6_from_frontend() {
        let p = ServicePeer::from_frontend("tcp://[::1]:15662").unwrap();
        assert_eq!(p.backend, "tcp://[::1]:15663");
    }

    #[test]
    fn ipc_backend_with_id() {
        let p =
            ServicePeer::from_backend("broker-b=ipc:///tmp/rb-b/service_bus_backend.ipc").unwrap();
        assert_eq!(p.backend, "ipc:///tmp/rb-b/service_bus_backend.ipc");
        assert_eq!(p.broker_id, "broker-b");
    }

    #[test]
    fn ipc_from_frontend_derives_backend_file() {
        let p = ServicePeer::from_frontend("ipc:///tmp/rb-b/service_bus_frontend.ipc").unwrap();
        assert_eq!(p.backend, "ipc:///tmp/rb-b/service_bus_backend.ipc");
    }
}
