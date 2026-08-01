//! Subscribe `/tf` + `/tf_static` into a shared [`Buffer`].

use super::buffer::Buffer;
use crate::tf2_msgs::msg::v1::TfMessage;
use crate::{Node, Result};
use std::sync::{Arc, Mutex};

/// Shared TF buffer updated by topic subscriptions.
pub type SharedBuffer = Arc<Mutex<Buffer>>;

/// Holds a buffer and the node subscriptions that feed it.
pub struct TfListener {
    buffer: SharedBuffer,
}

impl TfListener {
    /// Subscribe `tf_topic` (dynamic) and `tf_static_topic` (static).
    pub fn new(node: &mut Node, tf_topic: &str, tf_static_topic: &str) -> Result<Self> {
        let buffer: SharedBuffer = Arc::new(Mutex::new(Buffer::new()));

        let dyn_buf = Arc::clone(&buffer);
        node.create_subscription::<TfMessage, _>(
            tf_topic,
            move |_topic, msg| {
                if let Ok(mut guard) = dyn_buf.lock() {
                    guard.set_transform_msg(&msg, false);
                }
            },
            None,
        )?;

        let static_buf = Arc::clone(&buffer);
        node.create_subscription::<TfMessage, _>(
            tf_static_topic,
            move |_topic, msg| {
                if let Ok(mut guard) = static_buf.lock() {
                    guard.set_transform_msg(&msg, true);
                }
            },
            None,
        )?;

        Ok(Self { buffer })
    }

    /// Default topics `/tf` and `/tf_static`.
    pub fn with_defaults(node: &mut Node) -> Result<Self> {
        Self::new(node, "/tf", "/tf_static")
    }

    pub fn buffer(&self) -> SharedBuffer {
        Arc::clone(&self.buffer)
    }
}
