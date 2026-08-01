//! Encode `sensor_msgs/Image` to `foxglove_msgs/CompressedVideo` (H.264 / H.265).
//!
//! Enabled with Cargo feature `image-encoder` (on by default). Requires system FFmpeg.

pub mod codec;
pub mod config;
pub mod convert;
pub mod encoder;
pub mod node;

pub use config::EncoderConfig;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
