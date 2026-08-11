//! Topic QoS profile (v1).
//!
//! This is intentionally **not** a full ROS 2 / DDS QoS facsimile. For topics we only
//! honor a KeepLast-style depth, which maps to ZeroMQ high-water marks. Reliability is
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
    /// On a publisher this primarily sets send HWM; on a subscriber, receive HWM.
    /// Both directions are set to `depth` on the local socket.
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
}
