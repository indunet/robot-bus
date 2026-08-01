//! Capture USB / webcam frames and publish `sensor_msgs/Image` (`rgb8`).
//!
//! Enabled with Cargo feature `camera-capture` (on by default).

pub mod config;
pub mod device;
pub mod frame;
pub mod node;

pub use config::CaptureConfig;
pub use device::list_cameras;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
