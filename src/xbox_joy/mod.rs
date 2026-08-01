//! USB Xbox-layout joy → `XboxJoy` out, `XboxJoyRumble` in.
//!
//! Enabled with Cargo feature `xbox-joy` (on by default). Uses [gilrs]
//! (SDL GameController mappings) so standard Xbox USB receivers / pads work
//! without a custom driver on Linux, macOS, and Windows. Force feedback
//! (rumble) is supported on Linux and Windows; macOS input works but rumble
//! is not available in gilrs.

pub mod config;
pub mod device;
pub mod mapping;
pub mod node;
pub mod rumble;

pub use config::JoyConfig;
pub use device::list_joys;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
