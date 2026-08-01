//! Encode `sensor_msgs/Image` to `foxglove_msgs/CompressedVideo` (H.264 / H.265).

pub mod codec;
pub mod config;
pub mod convert;
pub mod encoder;
pub mod node;

pub use config::EncoderConfig;
pub use node::run;
