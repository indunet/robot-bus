//! Multicast UDP socket helpers.

use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

/// Socket that joins `multicast_addr` on `port` for receiving announces.
pub fn join_multicast_receiver(multicast_addr: Ipv4Addr, port: u16) -> io::Result<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    {
        let _ = socket.set_reuse_port(true);
    }
    socket.set_nonblocking(false)?;
    let bind_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    socket.bind(&SocketAddr::from(bind_addr).into())?;
    socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED)?;
    socket.set_multicast_loop_v4(true)?;
    Ok(socket.into())
}

/// Socket for sending announces to the multicast group.
pub fn multicast_sender(
    multicast_addr: Ipv4Addr,
    port: u16,
) -> io::Result<(UdpSocket, SocketAddr)> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&SocketAddr::from(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).into())?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_multicast_ttl_v4(1)?;
    let dest = SocketAddr::from(SocketAddrV4::new(multicast_addr, port));
    Ok((socket.into(), dest))
}

pub fn set_read_timeout(sock: &UdpSocket, timeout: Option<Duration>) -> io::Result<()> {
    sock.set_read_timeout(timeout)
}

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
