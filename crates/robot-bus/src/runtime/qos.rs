//! Topic and RPC QoS settings.

use crate::zmq_helpers::HighWaterMark;

pub use crate::wire::SubscriptionOverflowPolicy;

/// KeepLast history depth, with optional publisher-local topic retention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QosProfile {
    depth: i32,
    transient_local: bool,
}

impl QosProfile {
    /// Same depth as [`HighWaterMark::STREAM`] (8).
    pub const DEFAULT: Self = Self::keep_last(HighWaterMark::STREAM.snd);

    /// Keep the newest `depth` pending samples on a subscription.
    /// Native topic publishers and RPC endpoints use this depth as their socket HWM.
    pub const fn keep_last(depth: i32) -> Self {
        Self {
            depth,
            transient_local: false,
        }
    }

    /// Enable publisher-local retention for a native topic publisher or subscription.
    /// The publisher retains its latest sample for late joiners and reconnects.
    pub const fn transient_local(mut self) -> Self {
        self.transient_local = true;
        self
    }

    /// Whether publisher-local topic retention was requested.
    pub const fn is_transient_local(self) -> bool {
        self.transient_local
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

/// Server subscribe queue when `SubscribeRequest.qos_depth` is omitted (proto3 0).
/// Historical WS default; independent of ZMQ [`HighWaterMark::STREAM`].
pub(crate) const WS_SUBSCRIBE_QUEUE_DEFAULT: i32 = 64;
const WS_SUBSCRIBE_QUEUE_MAX: usize = 1_048_576;

/// Depth → bounded WebSocket subscription capacity.
///
/// `qos_depth <= 0` keeps [`WS_SUBSCRIBE_QUEUE_DEFAULT`].
#[doc(hidden)]
pub fn ws_subscribe_queue_capacity(qos_depth: i32) -> usize {
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
    fn keep_last_one_is_the_latest_message_profile() {
        assert_eq!(QosProfile::keep_last(1).depth(), 1);
        assert_eq!(
            SubscriptionOverflowPolicy::default(),
            SubscriptionOverflowPolicy::DropOldest
        );
    }
}
