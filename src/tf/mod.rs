//! Coordinate frame transforms (ROS TF2–compatible buffer / listener / broadcaster).
//!
//! Message truth source: `tf2_msgs/TFMessage` on `/tf` and `/tf_static`.
//!
//! Time semantics (v1): static edges always apply; dynamic edges use the latest
//! sample only (no interpolation or extrapolation).

pub mod broadcaster;
pub mod buffer;
pub mod convert;
pub mod error;
pub mod listener;
pub mod math;

pub use broadcaster::TransformBroadcaster;
pub use buffer::Buffer;
pub use convert::{
    make_tf_message, make_transform_stamped, msg_to_rigid, now_stamp, rigid_to_msg, stamped_to_rigid,
    static_stamp,
};
pub use error::TfError;
pub use listener::{SharedBuffer, TfListener};
pub use math::RigidTransform;
