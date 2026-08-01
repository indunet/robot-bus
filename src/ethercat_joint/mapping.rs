//! Map JointCommand / feedback ↔ per-joint setpoints.

use super::cia402::{self, DriveState};
use super::config::{JointConfig, JointMode};
use super::master::{JointFeedback, JointSetpoint};
use super::units;
use crate::robot_bus_interface::msg::v1::JointCommand;
use crate::sensor_msgs::msg::v1::JointState;
use crate::std_msgs::msg::v1::Header;

/// Latest SI setpoints held between bus messages (one per configured joint).
#[derive(Debug, Clone)]
pub struct CommandCache {
    pub position: Vec<Option<f64>>,
    pub velocity: Vec<Option<f64>>,
    pub effort: Vec<Option<f64>>,
}

impl CommandCache {
    pub fn new(n: usize) -> Self {
        Self {
            position: vec![None; n],
            velocity: vec![None; n],
            effort: vec![None; n],
        }
    }

    pub fn apply_command(&mut self, joints: &[JointConfig], cmd: &JointCommand) {
        for (i, name) in cmd.joint_names.iter().enumerate() {
            let Some(ji) = joints.iter().position(|j| j.name == *name) else {
                continue;
            };
            if i < cmd.position.len() {
                self.position[ji] = Some(cmd.position[i]);
            }
            if i < cmd.velocity.len() {
                self.velocity[ji] = Some(cmd.velocity[i]);
            }
            if i < cmd.effort.len() {
                self.effort[ji] = Some(cmd.effort[i]);
            }
        }
    }
}

pub fn build_setpoints(
    joints: &[JointConfig],
    cache: &CommandCache,
    feedback: &[JointFeedback],
    want_enabled: bool,
    pulse_fault_reset: bool,
) -> Vec<JointSetpoint> {
    let mut out = Vec::with_capacity(joints.len());
    for (i, j) in joints.iter().enumerate() {
        let sw = feedback.get(i).map(|f| f.statusword).unwrap_or(0);
        let controlword = cia402::next_controlword(sw, want_enabled, pulse_fault_reset && i == 0);
        let target = match j.mode {
            JointMode::Csp => {
                let rad = cache.position[i].unwrap_or_else(|| {
                    feedback
                        .get(i)
                        .map(|f| units::ticks_to_rad(j, f.actual))
                        .unwrap_or(0.0)
                });
                units::rad_to_ticks(j, rad)
            }
            JointMode::Csv => {
                let rad_s = cache.velocity[i].unwrap_or(0.0);
                units::rad_s_to_ticks_s(j, rad_s)
            }
            JointMode::Cst => {
                let nm = cache.effort[i].unwrap_or(0.0);
                units::nm_to_torque_cmd(j, nm)
            }
        };
        out.push(JointSetpoint {
            target,
            controlword,
        });
    }
    out
}

pub fn feedback_to_joint_state(
    joints: &[JointConfig],
    feedback: &[JointFeedback],
    frame_id: &str,
    stamp_sec: i32,
    stamp_nanosec: u32,
) -> JointState {
    let mut name = Vec::with_capacity(joints.len());
    let mut position = Vec::with_capacity(joints.len());
    let mut velocity = Vec::with_capacity(joints.len());
    let mut effort = Vec::with_capacity(joints.len());

    for (i, j) in joints.iter().enumerate() {
        name.push(j.name.clone());
        let fb = feedback.get(i).copied().unwrap_or_default();
        match j.mode {
            JointMode::Csp => {
                position.push(units::ticks_to_rad(j, fb.actual));
                velocity.push(0.0);
                effort.push(0.0);
            }
            JointMode::Csv => {
                position.push(0.0);
                velocity.push(units::ticks_s_to_rad_s(j, fb.actual));
                effort.push(0.0);
            }
            JointMode::Cst => {
                position.push(0.0);
                velocity.push(0.0);
                effort.push(units::torque_cmd_to_nm(j, fb.actual));
            }
        }
    }

    JointState {
        header: Some(Header {
            stamp: Some(crate::builtin_interfaces::msg::v1::Time {
                sec: stamp_sec,
                nanosec: stamp_nanosec,
            }),
            frame_id: frame_id.to_string(),
        }),
        name,
        position,
        velocity,
        effort,
    }
}

pub fn any_fault(feedback: &[JointFeedback]) -> bool {
    feedback
        .iter()
        .any(|f| DriveState::from_statusword(f.statusword) == DriveState::Fault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ethercat_joint::config::PdoOffsets;

    #[test]
    fn apply_position_command() {
        let joints = [JointConfig {
            name: "j1".into(),
            station_address: 1,
            mode: JointMode::Csp,
            encoder_ticks_per_rev: 1000.0,
            gear_ratio: 1.0,
            direction: 1,
            position_offset_rad: 0.0,
            pdo: PdoOffsets::default(),
        }];
        let mut cache = CommandCache::new(1);
        cache.apply_command(
            &joints,
            &JointCommand {
                header: None,
                joint_names: vec!["j1".into()],
                position: vec![1.0],
                velocity: vec![],
                effort: vec![],
            },
        );
        assert_eq!(cache.position[0], Some(1.0));
    }
}
