//! Extension hooks for secondary development.
//!
//! # Example
//!
//! ```ignore
//! use robot_bus::ethercat_joint::{run_with_hooks, FaultAction, JointHooks, JointCommand};
//! use robot_bus::NodeOptions;
//!
//! struct ClampHooks;
//!
//! impl JointHooks for ClampHooks {
//!     fn on_command(&mut self, mut cmd: JointCommand) -> JointCommand {
//!         for p in &mut cmd.position {
//!             *p = p.clamp(-1.0, 1.0);
//!         }
//!         cmd
//!     }
//!
//!     fn on_fault(&mut self, joint_name: &str, statusword: u16) -> FaultAction {
//!         eprintln!("fault on {joint_name}: {statusword:#06x}");
//!         FaultAction::Disable
//!     }
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     run_with_hooks(
//!         "my_joints",
//!         NodeOptions::tcp_at("localhost"),
//!         Some("robot.yaml"),
//!         ClampHooks,
//!     )
//! }
//! ```
//!
//! PP / PV / Homing and other non-cyclic CiA402 modes are intentionally unsupported;
//! implement them in a custom binary if needed.

use crate::robot_bus_interface::msg::v1::JointCommand;

/// What to do when a drive reports a fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultAction {
    /// Request CiA402 fault reset pulse on the next cycles.
    Reset,
    /// Leave the drive in fault; publish diagnostics only.
    Ignore,
    /// Clear want-enabled (safe stop).
    Disable,
}

/// Optional hooks for vendors / integrators.
///
/// Use [`crate::ethercat_joint::run_with_hooks`] from a custom binary to inject behaviour
/// without forking the whole node.
pub trait JointHooks: Send {
    /// Transform an inbound command (limits, calibration, frame changes).
    fn on_command(&mut self, cmd: JointCommand) -> JointCommand {
        cmd
    }

    /// Decide how to handle a faulting joint (`joint_name`, `statusword`).
    fn on_fault(&mut self, _joint_name: &str, _statusword: u16) -> FaultAction {
        FaultAction::Disable
    }
}

/// Default no-op hooks used by `rbus_ethercat_joint`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopHooks;

impl JointHooks for NoopHooks {}
