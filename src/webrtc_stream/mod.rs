//! Subscribe to Image / RawAudio / data topics and serve a WHEP livestream.
//!
//! Enabled with Cargo feature `webrtc` (off by default). Needs system FFmpeg + libopus.

pub mod config;
pub mod hub;
pub mod media;
pub mod node;
pub mod whep;

pub use config::WebrtcConfig;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
