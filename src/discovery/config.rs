//! Discovery defaults and broker/client options.

use std::net::Ipv4Addr;
use std::time::Duration;

/// Magic string embedded in every valid [`super::BrokerAnnouncement`].
pub const MAGIC: &str = "RBUS";

/// Supported announce schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Default multicast group (away from ROS2 / DDS `239.255.0.1`).
pub const DEFAULT_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 76, 67);

/// Default UDP discovery port (near robot-bus data plane ports; away from DDS 7400).
pub const DEFAULT_DISCOVERY_PORT: u16 = 15550;

/// Default announce interval.
pub const DEFAULT_DISCOVERY_INTERVAL: Duration = Duration::from_secs(1);

/// Default client wait timeout.
pub const DEFAULT_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);

/// Broker-side discovery / announce settings.
#[derive(Clone, Debug)]
pub struct DiscoveryConfig {
    /// When false, the broker does not send UDP announces.
    pub enabled: bool,
    pub domain_id: u32,
    pub multicast_addr: Ipv4Addr,
    pub multicast_port: u16,
    pub interval: Duration,
    /// Override advertise host (otherwise inferred / `127.0.0.1`).
    pub advertise_host: Option<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            domain_id: 0,
            multicast_addr: DEFAULT_MULTICAST_ADDR,
            multicast_port: DEFAULT_DISCOVERY_PORT,
            interval: DEFAULT_DISCOVERY_INTERVAL,
            advertise_host: None,
        }
    }
}

/// Client-side wait options.
#[derive(Clone, Debug)]
pub struct DiscoverOpts {
    pub domain_id: u32,
    /// When set, only accept this broker id.
    pub broker_id: Option<String>,
    pub multicast_addr: Ipv4Addr,
    pub multicast_port: u16,
    pub timeout: Duration,
}

impl Default for DiscoverOpts {
    fn default() -> Self {
        Self {
            domain_id: 0,
            broker_id: None,
            multicast_addr: DEFAULT_MULTICAST_ADDR,
            multicast_port: DEFAULT_DISCOVERY_PORT,
            timeout: DEFAULT_DISCOVERY_TIMEOUT,
        }
    }
}
