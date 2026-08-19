//! Topic QoS profile (v1).
//!
//! This is intentionally **not** a full ROS 2 / DDS QoS facsimile. For topics we only
//! honor a KeepLast-style depth. On ZMQ nodes that maps to socket high-water marks.
//! On WebSocket nodes, subscribe depth sizes the gateway→client queue (drop-on-full);
//! publish QoS is ignored because WS publishers share one gateway PUB. Reliability is
//! fixed to best-effort (PUB/SUB has no ACK). Service and action ignore this type for now.

use crate::zmq_helpers::HighWaterMark;

/// Topic QoS: KeepLast depth → ZMQ HWM. Service / action do not use this yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QosProfile {
    depth: i32,
}

impl QosProfile {
    /// Same depth as [`HighWaterMark::STREAM`] (8).
    pub const DEFAULT: Self = Self::keep_last(HighWaterMark::STREAM.snd);

    /// ROS 2–style `KeepLast(depth)` history depth.
    ///
    /// On a ZMQ publisher this primarily sets send HWM; on a ZMQ subscriber, receive HWM.
    /// Both directions are set to `depth` on the local socket. On a WebSocket node,
    /// subscribe depth sizes the gateway→client queue; publish depth is ignored.
    pub const fn keep_last(depth: i32) -> Self {
        Self { depth }
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

/// KeepLast depth → WebSocket subscribe mpsc capacity (drop-on-full).
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
}
