//! Encode / decode / validate discovery protobuf datagrams.

use prost::Message;

use crate::errors::{BusError, Result};
use crate::generated::robot_bus_interface::msg::v1::{BrokerAnnounce, TcpPorts};

use super::config::{MAGIC, SCHEMA_VERSION};

/// Validated announce ready for [`super::BrokerAnnouncement::apply`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerAnnouncement {
    pub broker_id: String,
    pub domain_id: u32,
    pub advertise_host: String,
    pub tcp: Option<TcpPorts>,
    pub ipc_dir: Option<String>,
    pub inproc_prefix: Option<String>,
    pub grpc_url: Option<String>,
    pub console_url: Option<String>,
}

impl From<&BrokerAnnouncement> for BrokerAnnounce {
    fn from(a: &BrokerAnnouncement) -> Self {
        BrokerAnnounce {
            magic: MAGIC.to_string(),
            schema_version: SCHEMA_VERSION,
            broker_id: a.broker_id.clone(),
            domain_id: a.domain_id,
            advertise_host: a.advertise_host.clone(),
            tcp: a.tcp,
            ipc_dir: a.ipc_dir.clone(),
            inproc_prefix: a.inproc_prefix.clone(),
            grpc_url: a.grpc_url.clone(),
            console_url: a.console_url.clone(),
        }
    }
}

/// Encode a validated announcement to a UDP payload.
pub fn encode_announce(ann: &BrokerAnnouncement) -> Result<Vec<u8>> {
    let msg = BrokerAnnounce::from(ann);
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf)
        .map_err(|e| BusError::Protocol(format!("encode BrokerAnnounce: {e}")))?;
    Ok(buf)
}

/// Decode + validate a UDP payload. Errors are invalid packets.
pub fn decode_announce(bytes: &[u8]) -> Result<BrokerAnnouncement> {
    let msg = BrokerAnnounce::decode(bytes)
        .map_err(|e| BusError::Protocol(format!("decode BrokerAnnounce: {e}")))?;
    validate_message(msg)
}

/// Like [`decode_announce`], but returns `Ok(None)` for invalid/garbage datagrams.
pub fn try_parse_datagram(bytes: &[u8]) -> Option<BrokerAnnouncement> {
    decode_announce(bytes).ok()
}

fn validate_message(msg: BrokerAnnounce) -> Result<BrokerAnnouncement> {
    if msg.magic != MAGIC {
        return Err(BusError::Protocol(format!(
            "invalid discovery magic {:?}, expected {MAGIC:?}",
            msg.magic
        )));
    }
    if msg.schema_version != SCHEMA_VERSION {
        return Err(BusError::Protocol(format!(
            "unsupported discovery schema_version {}, expected {SCHEMA_VERSION}",
            msg.schema_version
        )));
    }
    if msg.broker_id.is_empty() {
        return Err(BusError::Protocol(
            "discovery announce missing broker_id".into(),
        ));
    }
    if msg.advertise_host.is_empty()
        || msg.advertise_host == "0.0.0.0"
        || msg.advertise_host == "*"
    {
        return Err(BusError::Protocol(format!(
            "discovery announce has non-connectable advertise_host {:?}",
            msg.advertise_host
        )));
    }
    let tcp = msg.tcp.ok_or_else(|| {
        BusError::Protocol("discovery announce missing tcp ports".into())
    })?;
    if tcp.message_xsub == 0
        || tcp.message_xpub == 0
        || tcp.service_frontend == 0
        || tcp.service_backend == 0
        || tcp.action_frontend == 0
        || tcp.action_backend == 0
    {
        return Err(BusError::Protocol(
            "discovery announce has zero tcp port".into(),
        ));
    }
    Ok(BrokerAnnouncement {
        broker_id: msg.broker_id,
        domain_id: msg.domain_id,
        advertise_host: msg.advertise_host,
        tcp: Some(tcp),
        ipc_dir: nonempty(msg.ipc_dir),
        inproc_prefix: nonempty(msg.inproc_prefix),
        grpc_url: nonempty(msg.grpc_url),
        console_url: nonempty(msg.console_url),
    })
}

fn nonempty(v: Option<String>) -> Option<String> {
    v.filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::robot_bus_interface::msg::v1::TcpPorts;

    fn sample() -> BrokerAnnouncement {
        BrokerAnnouncement {
            broker_id: "broker-a".into(),
            domain_id: 0,
            advertise_host: "127.0.0.1".into(),
            tcp: Some(TcpPorts {
                message_xsub: 15560,
                message_xpub: 15561,
                service_frontend: 15662,
                service_backend: 15663,
                action_frontend: 15664,
                action_backend: 15665,
            }),
            ipc_dir: Some("/tmp/robot_bus".into()),
            inproc_prefix: Some("robot_bus".into()),
            grpc_url: Some("http://127.0.0.1:15770".into()),
            console_url: Some("http://127.0.0.1:15771".into()),
        }
    }

    #[test]
    fn round_trip() {
        let ann = sample();
        let bytes = encode_announce(&ann).unwrap();
        let got = decode_announce(&bytes).unwrap();
        assert_eq!(got, ann);
    }

    #[test]
    fn garbage_bytes_rejected() {
        assert!(decode_announce(b"not-a-proto").is_err());
        assert!(try_parse_datagram(b"\x00\x01\x02").is_none());
    }

    #[test]
    fn wrong_magic_rejected() {
        let mut msg = BrokerAnnounce::from(&sample());
        msg.magic = "NOPE".into();
        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();
        assert!(decode_announce(&buf).is_err());
    }

    #[test]
    fn wrong_version_rejected() {
        let mut msg = BrokerAnnounce::from(&sample());
        msg.schema_version = 99;
        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();
        assert!(decode_announce(&buf).is_err());
    }
}
