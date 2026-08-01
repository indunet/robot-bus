//! Thin helper to publish `tf2_msgs/TFMessage`.

use super::convert::{make_tf_message, make_transform_stamped};
use super::math::RigidTransform;
use crate::builtin_interfaces::msg::v1::Time;
use crate::geometry_msgs::msg::v1::TransformStamped;
use crate::tf2_msgs::msg::v1::TfMessage;
use crate::{Result, TopicPublisher};

/// Publishes transforms as `TFMessage` batches.
pub struct TransformBroadcaster {
    pub publisher: TopicPublisher<TfMessage>,
}

impl TransformBroadcaster {
    pub fn new(publisher: TopicPublisher<TfMessage>) -> Self {
        Self { publisher }
    }

    pub fn send_transform(&self, stamped: TransformStamped) -> Result<()> {
        self.send_transforms(vec![stamped])
    }

    pub fn send_transforms(&self, transforms: Vec<TransformStamped>) -> Result<()> {
        self.publisher.publish(&make_tf_message(transforms))
    }

    pub fn send_rigid(
        &self,
        parent: impl Into<String>,
        child: impl Into<String>,
        transform: RigidTransform,
        stamp: Time,
    ) -> Result<()> {
        self.send_transform(make_transform_stamped(parent, child, transform, stamp))
    }
}
