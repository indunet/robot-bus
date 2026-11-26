//! Explicit host overrides and TCP bind parsing.

#[cfg(feature = "broker")]
use super::config::DiscoveryConfig;

#[cfg(feature = "broker")]
pub(crate) fn explicit_advertise_host(config: &DiscoveryConfig) -> Option<&str> {
    config
        .advertise_host
        .as_deref()
        .map(str::trim)
        .filter(|s| !matches!(*s, "" | "0.0.0.0" | "*" | "::" | "[::]"))
}

/// True if the TCP listener does not bind a wildcard address.
#[cfg(feature = "broker")]
pub(crate) fn is_specific_tcp_bind(bind: &str) -> bool {
    bind.strip_prefix("tcp://")
        .and_then(|addr| addr.rsplit_once(':'))
        .is_some_and(|(host, _)| !matches!(host, "*" | "0.0.0.0" | "[::]"))
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
