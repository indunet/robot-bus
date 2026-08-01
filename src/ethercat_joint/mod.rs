//! EtherCAT / CiA402 joint tool node → `JointState` out, `JointCommand` in.
//!
//! Enabled with Cargo feature `ethercat-joint` (**off by default**). Uses
//! [ethercrab](https://crates.io/crates/ethercrab) for the real master, or
//! `backend: mock` for hardware-free bring-up and tests.
//!
//! # Secondary development
//!
//! Depend on `robot-bus` with `features = ["ethercat-joint"]` and call
//! [`run_with_hooks`] with a custom [`hooks::JointHooks`] implementation from your
//! own binary. The stock CLI [`crate` binary `rbus_ethercat_joint`] uses [`hooks::NoopHooks`].

pub mod cia402;
pub mod config;
pub mod hooks;
pub mod mapping;
pub mod master;
pub mod node;
pub mod units;

pub use config::{BackendKind, EthercatJointConfig, JointConfig, JointMode};
pub use hooks::{FaultAction, JointHooks, NoopHooks};
pub use master::{create_master, EthercatMaster, MockMaster};
pub use node::{list_devices, run, run_with_hooks};

/// Example YAML embedded for `--print-example-config`.
pub const EXAMPLE_CONFIG: &str = include_str!("example.yaml");
