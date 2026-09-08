//! Topic / RPC QoS profile (v1).
//!
//! This is intentionally **not** a full ROS 2 / DDS QoS facsimile. We only
//! map KeepLast-style depth to ZeroMQ high-water marks, with explicit
//! overflow replacement policies for WebSocket subscriptions.
//! On ZMQ topic sockets that is PUB/SUB HWM; on ZMQ service / action sockets
//! it is DEALER HWM (`snd` / `rcv` both = depth). On WebSocket nodes, subscribe
//! depth sizes the gateway→client queue (default: drop newest on full); publish QoS is ignored
//! because WS publishers share one gateway PUB. Reliability is fixed to
//! best-effort (PUB/SUB has no ACK; RPC has no DDS reliability either).

use crate::zmq_helpers::HighWaterMark;

/// Overflow policy for a WebSocket subscription's pending gateway messages.
/// Does not recall messages already written to the network or guarantee delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SubscriptionOverflowPolicy {
    #[default]
    DropNewest = 0,
    DropOldest = 1,
    Latest = 2,
}

impl SubscriptionOverflowPolicy {
    pub fn from_wire(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::DropNewest),
            1 => Some(Self::DropOldest),
            2 => Some(Self::Latest),
            _ => None,
        }
    }
}

/// KeepLast depth → ZMQ HWM; optional replacement policy for WS subscriptions.
/// `latest` and `keep_recent` affect subscriptions only. Native subscriptions
/// reject these policies; publishers and RPC clients use only the depth mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QosProfile {
    depth: i32,
    overflow: SubscriptionOverflowPolicy,
}

impl QosProfile {
    /// Same depth as [`HighWaterMark::STREAM`] (8).
    pub const DEFAULT: Self = Self::keep_last(HighWaterMark::STREAM.snd);

    /// ROS 2–style `KeepLast(depth)` history depth.
    ///
    /// On a ZMQ publisher this primarily sets send HWM; on a ZMQ subscriber, receive HWM.
    /// Both directions are set to `depth` on the local socket. Service / action DEALER
    /// sockets use the same mapping. On a WebSocket node, subscribe depth sizes the
    /// gateway→client queue; publish / RPC depth is ignored.
    pub const fn keep_last(depth: i32) -> Self {
        Self {
            depth,
            overflow: SubscriptionOverflowPolicy::DropNewest,
        }
    }

    /// Keep only the latest pending message per subscription filter (WebSocket only).
    pub const fn latest() -> Self {
        Self {
            depth: 1,
            overflow: SubscriptionOverflowPolicy::Latest,
        }
    }

    /// Keep the most recent N pending messages, evicting the oldest (WebSocket only).
    pub const fn keep_recent(depth: i32) -> Self {
        Self {
            depth,
            overflow: SubscriptionOverflowPolicy::DropOldest,
        }
    }

    pub const fn overflow_policy(self) -> SubscriptionOverflowPolicy {
        self.overflow
    }

    /// History depth (KeepLast N).
    pub const fn depth(self) -> i32 {
        self.depth
    }

    /// Map to ZMQ HWM (`snd` / `rcv` both = depth).
    pub const fn to_hwm(self) -> HighWaterMark {
        HighWaterMark::new(self.depth, self.depth)
    }
}

impl Default for QosProfile {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Gateway subscribe queue when `SubscribeRequest.qos_depth` is omitted (proto3 0).
/// Historical WS default; independent of ZMQ [`HighWaterMark::STREAM`].
pub(crate) const WS_SUBSCRIBE_QUEUE_DEFAULT: i32 = 64;
const WS_SUBSCRIBE_QUEUE_MAX: usize = 1_048_576;

/// Depth → bounded WebSocket subscription capacity.
///
/// `qos_depth <= 0` keeps [`WS_SUBSCRIBE_QUEUE_DEFAULT`].
pub(crate) fn ws_subscribe_queue_capacity(qos_depth: i32) -> usize {
    if qos_depth <= 0 {
        WS_SUBSCRIBE_QUEUE_DEFAULT as usize
    } else {
        (qos_depth as usize).clamp(1, WS_SUBSCRIBE_QUEUE_MAX)
    }
}

/// Alias matching rclrs-style naming (`QOS_PROFILE_DEFAULT`).
pub const QOS_PROFILE_DEFAULT: QosProfile = QosProfile::DEFAULT;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_last_maps_to_hwm() {
        let qos = QosProfile::keep_last(16);
        assert_eq!(qos.depth(), 16);
        assert_eq!(qos.to_hwm(), HighWaterMark::new(16, 16));
        assert_eq!(QosProfile::DEFAULT.depth(), HighWaterMark::STREAM.snd);
    }

    #[test]
    fn ws_subscribe_queue_honors_keep_last_and_default() {
        assert_eq!(ws_subscribe_queue_capacity(0), 64);
        assert_eq!(ws_subscribe_queue_capacity(-1), 64);
        assert_eq!(ws_subscribe_queue_capacity(10), 10);
        assert_eq!(ws_subscribe_queue_capacity(1), 1);
        assert_eq!(ws_subscribe_queue_capacity(i32::MAX), 1_048_576);
    }

    #[test]
    fn native_subscription_rejects_unsupported_replacement_policy() {
        let mut node = crate::Node::new("native-qos");
        for qos in [QosProfile::latest(), QosProfile::keep_recent(3)] {
            let error = node
                .create_subscription_raw_with_qos("/sample", qos, std::sync::Arc::new(|_| {}), None)
                .unwrap_err();
            assert!(error.to_string().contains("require WebSocket transport"));
        }
    }
}
