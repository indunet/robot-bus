//! Capture microphone PCM and publish `foxglove_msgs/RawAudio` (`pcm-s16`).
//!
//! Enabled with Cargo feature `audio-capture` (on by default).

pub mod config;
pub mod device;
pub mod node;
pub mod pcm;

pub use config::CaptureConfig;
pub use device::list_input_devices;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
