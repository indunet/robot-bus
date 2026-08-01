//! In-process mock master for tests and hardware-free bring-up.

use super::{EthercatMaster, JointFeedback, JointSetpoint, SlaveInfo};
use crate::ethercat_joint::cia402::{self, DriveState};
use crate::ethercat_joint::config::JointConfig;
use anyhow::Result;

pub struct MockMaster {
    joints: Vec<JointConfig>,
    actual: Vec<i32>,
    statusword: Vec<u16>,
    want_enabled: bool,
    pulse_fault_reset: bool,
}

impl MockMaster {
    pub fn new() -> Self {
        Self {
            joints: Vec::new(),
            actual: Vec::new(),
            statusword: Vec::new(),
            want_enabled: false,
            pulse_fault_reset: false,
        }
    }
}

impl Default for MockMaster {
    fn default() -> Self {
        Self::new()
    }
}

impl EthercatMaster for MockMaster {
    fn configure(&mut self, joints: &[JointConfig]) -> Result<()> {
        self.joints = joints.to_vec();
        self.actual = vec![0; joints.len()];
        // Start in Switch on disabled.
        self.statusword = vec![cia402::status::SWITCH_ON_DISABLED; joints.len()];
        Ok(())
    }

    fn list_slaves(&self) -> Vec<SlaveInfo> {
        self.joints
            .iter()
            .map(|j| SlaveInfo {
                configured_address: j.station_address,
                name: format!("mock:{}", j.name),
            })
            .collect()
    }

    fn set_want_enabled(&mut self, enabled: bool) {
        self.want_enabled = enabled;
    }

    fn request_fault_reset(&mut self) {
        self.pulse_fault_reset = true;
    }

    fn cycle(&mut self, setpoints: &[JointSetpoint], feedback: &mut [JointFeedback]) -> Result<()> {
        let n = self.joints.len().min(setpoints.len()).min(feedback.len());
        for i in 0..n {
            let cw = if self.pulse_fault_reset {
                cia402::control::FAULT_RESET
            } else {
                setpoints[i].controlword
            };
            // Advance mock state machine roughly like a drive.
            let state = DriveState::from_statusword(self.statusword[i]);
            if cw & cia402::control::FAULT_RESET != 0 {
                self.statusword[i] = cia402::status::SWITCH_ON_DISABLED;
            } else {
                self.statusword[i] = match state {
                    DriveState::SwitchOnDisabled | DriveState::NotReady | DriveState::Unknown
                        if cw == cia402::control::SHUTDOWN =>
                    {
                        0x0021 // Ready to switch on
                    }
                    DriveState::ReadyToSwitchOn if cw == cia402::control::SWITCH_ON_CMD => 0x0023,
                    DriveState::SwitchedOn | DriveState::QuickStopActive
                        if cw == cia402::control::ENABLE_OP =>
                    {
                        0x0027
                    }
                    DriveState::OperationEnabled if cw == cia402::control::ENABLE_OP => 0x0027,
                    DriveState::OperationEnabled if cw == cia402::control::DISABLE_OP => 0x0023,
                    other => {
                        let _ = other;
                        self.statusword[i]
                    }
                };
            }

            if DriveState::from_statusword(self.statusword[i]) == DriveState::OperationEnabled {
                // Track target loosely for CSP/CSV/CST (all use `target` field).
                self.actual[i] = setpoints[i].target;
            }

            feedback[i] = JointFeedback {
                actual: self.actual[i],
                statusword: self.statusword[i],
                online: true,
            };
        }
        self.pulse_fault_reset = false;
        let _ = self.want_enabled;
        Ok(())
    }

    fn shutdown(&mut self) {
        for sw in &mut self.statusword {
            *sw = cia402::status::SWITCH_ON_DISABLED;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ethercat_joint::config::{JointMode, PdoOffsets};

    #[test]
    fn mock_enable_and_track() {
        let mut m = MockMaster::new();
        let joints = [JointConfig {
            name: "j1".into(),
            station_address: 1001,
            mode: JointMode::Csp,
            encoder_ticks_per_rev: 1000.0,
            gear_ratio: 1.0,
            direction: 1,
            position_offset_rad: 0.0,
            pdo: PdoOffsets::default(),
        }];
        m.configure(&joints).unwrap();
        m.set_want_enabled(true);
        let mut fb = [JointFeedback::default()];
        // Shutdown
        m.cycle(
            &[JointSetpoint {
                target: 0,
                controlword: cia402::control::SHUTDOWN,
            }],
            &mut fb,
        )
        .unwrap();
        assert_eq!(fb[0].statusword, 0x0021);
        m.cycle(
            &[JointSetpoint {
                target: 0,
                controlword: cia402::control::SWITCH_ON_CMD,
            }],
            &mut fb,
        )
        .unwrap();
        assert_eq!(fb[0].statusword, 0x0023);
        m.cycle(
            &[JointSetpoint {
                target: 42,
                controlword: cia402::control::ENABLE_OP,
            }],
            &mut fb,
        )
        .unwrap();
        assert_eq!(fb[0].statusword, 0x0027);
        assert_eq!(fb[0].actual, 42);
    }
}
