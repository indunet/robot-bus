//! Subscribe to `foxglove_msgs/RawAudio` (`pcm-s16`) and play on a speaker.
//!
//! Enabled with Cargo feature `audio-play` (on by default).

pub mod config;
pub mod device;
pub mod node;
pub mod pcm;

pub use config::PlayConfig;
pub use device::list_output_devices;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
