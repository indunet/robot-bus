//! EtherCAT master abstraction.

mod mock;

#[cfg(feature = "ethercat-joint")]
mod ethercrab_backend;

use super::config::JointConfig;
use anyhow::Result;

pub use mock::MockMaster;

#[cfg(feature = "ethercat-joint")]
pub use ethercrab_backend::EthercrabMaster;

/// Per-cycle setpoint written to a drive (already in drive units).
#[derive(Debug, Clone, Copy, Default)]
pub struct JointSetpoint {
    pub target: i32,
    pub controlword: u16,
}

/// Per-cycle feedback read from a drive (drive units).
#[derive(Debug, Clone, Copy, Default)]
pub struct JointFeedback {
    pub actual: i32,
    pub statusword: u16,
    pub online: bool,
}

#[derive(Debug, Clone)]
pub struct SlaveInfo {
    pub configured_address: u16,
    pub name: String,
}

/// Pluggable EtherCAT / mock backend.
pub trait EthercatMaster: Send {
    fn configure(&mut self, joints: &[JointConfig]) -> Result<()>;
    fn list_slaves(&self) -> Vec<SlaveInfo>;
    fn set_want_enabled(&mut self, enabled: bool);
    fn request_fault_reset(&mut self);
    fn cycle(&mut self, setpoints: &[JointSetpoint], feedback: &mut [JointFeedback]) -> Result<()>;
    fn shutdown(&mut self);
}

/// Build the configured backend.
pub fn create_master(backend: super::config::BackendKind, iface: &str) -> Result<Box<dyn EthercatMaster>> {
    match backend {
        super::config::BackendKind::Mock => Ok(Box::new(MockMaster::new())),
        super::config::BackendKind::Ethercrab => {
            #[cfg(feature = "ethercat-joint")]
            {
                Ok(Box::new(EthercrabMaster::new(iface)?))
            }
            #[cfg(not(feature = "ethercat-joint"))]
            {
                let _ = iface;
                anyhow::bail!("ethercrab backend requires feature `ethercat-joint`")
            }
        }
    }
}
