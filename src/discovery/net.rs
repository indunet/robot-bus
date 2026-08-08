//! Host inference and TCP bind parsing.

use std::net::UdpSocket;

use super::config::DiscoveryConfig;

/// Best-effort non-loopback IPv4 for advertise_host; falls back to 127.0.0.1.
pub fn infer_advertise_host() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        // Destination need not be reachable; used only to pick a local interface.
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local) = socket.local_addr() {
                let ip = local.ip();
                if let std::net::IpAddr::V4(v4) = ip {
                    if !v4.is_loopback() && !v4.is_unspecified() {
                        return v4.to_string();
                    }
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Resolve advertise host from config or inference.
pub fn resolve_advertise_host(config: &DiscoveryConfig) -> String {
    config
        .advertise_host
        .clone()
        .filter(|s| !s.is_empty() && s != "0.0.0.0" && s != "*")
        .unwrap_or_else(infer_advertise_host)
}

/// Parse TCP port from a ZMQ bind endpoint (`tcp://host:port`).
pub fn tcp_port_from_bind(bind: &str) -> Option<u16> {
    let rest = bind.strip_prefix("tcp://")?;
    let port_str = if let Some(idx) = rest.rfind(']') {
        // tcp://[::1]:15560
        rest.get(idx + 1..)?.strip_prefix(':')?
    } else {
        rest.rsplit_once(':')?.1
    };
    port_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ipv4_bind_port() {
        assert_eq!(tcp_port_from_bind("tcp://0.0.0.0:15560"), Some(15560));
        assert_eq!(tcp_port_from_bind("tcp://127.0.0.1:16561"), Some(16561));
    }

    #[test]
    fn parses_ipv6_bind_port() {
        assert_eq!(tcp_port_from_bind("tcp://[::1]:15560"), Some(15560));
    }
}
