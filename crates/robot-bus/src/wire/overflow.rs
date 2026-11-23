//! Overflow policy for a WebSocket subscription's pending server messages.
//!
//! Shared by the frame codec and the Node runtime so neither module depends
//! on the other. Does not recall messages already written to the network or
//! guarantee delivery.

/// Overflow policy for a WebSocket subscription's pending server messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SubscriptionOverflowPolicy {
    /// Compatibility for explicitly requested wire policy 0.
    DropNewest = 0,
    #[default]
    DropOldest = 1,
    /// Compatibility for wire policy 2; equivalent to KeepLast(1).
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
