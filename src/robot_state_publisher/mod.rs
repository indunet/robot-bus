//! URDF + JointState → dynamic `/tf` and fixed `/tf_static`.
//!
//! Enabled with Cargo feature `robot-state-publisher` (on by default).
//! Supports joint types: fixed, revolute, continuous, prismatic, and URDF `<mimic>`.

pub mod config;
pub mod kinematics;
pub mod node;

pub use config::RobotStatePublisherConfig;
pub use node::run;

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");

/// Minimal 2-DOF test URDF (also used by unit tests).
pub const EXAMPLE_URDF: &str = include_str!("examples/simple_arm.urdf");
