//! Publish static transforms from YAML to `/tf_static`.
//!
//! Enabled with Cargo feature `static-transform-publisher` (on by default).

pub mod config;
pub mod node;

pub use config::StaticTransformConfig;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
