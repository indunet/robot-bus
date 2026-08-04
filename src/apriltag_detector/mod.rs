//! Detect AprilTags in `sensor_msgs/Image` and publish `apriltag_msgs/AprilTagDetectionArray`.
//!
//! Enabled with Cargo feature `apriltag-detector` (on by default). Links the AprilTag C
//! library via the [`apriltag`] crate (statically by default).

pub mod config;
pub mod convert;
pub mod detector;
pub mod node;

pub use config::DetectorConfig;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
