//! Decode `foxglove_msgs/CompressedVideo` (H.264 / H.265) to `sensor_msgs/Image`.
//!
//! Enabled with Cargo feature `image-decoder` (on by default). Requires system FFmpeg.

pub mod codec;
pub mod config;
pub mod convert;
pub mod decoder;
pub mod node;

pub use config::DecoderConfig;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
