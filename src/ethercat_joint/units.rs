//! Unit conversion between bus SI units and drive encoder units.

use super::config::JointConfig;

/// Convert joint position (rad) to encoder ticks.
pub fn rad_to_ticks(joint: &JointConfig, rad: f64) -> i32 {
    let motor_rad =
        (rad - joint.position_offset_rad) * f64::from(joint.direction) * joint.gear_ratio;
    let ticks = motor_rad / (std::f64::consts::TAU) * joint.encoder_ticks_per_rev;
    ticks.round() as i32
}

/// Encoder ticks → joint position (rad).
pub fn ticks_to_rad(joint: &JointConfig, ticks: i32) -> f64 {
    let motor_rad = f64::from(ticks) / joint.encoder_ticks_per_rev * std::f64::consts::TAU;
    motor_rad / joint.gear_ratio * f64::from(joint.direction) + joint.position_offset_rad
}

/// Joint velocity (rad/s) → encoder ticks/s.
pub fn rad_s_to_ticks_s(joint: &JointConfig, rad_s: f64) -> i32 {
    let motor_rad_s = rad_s * f64::from(joint.direction) * joint.gear_ratio;
    let ticks_s = motor_rad_s / (std::f64::consts::TAU) * joint.encoder_ticks_per_rev;
    ticks_s.round() as i32
}

/// Encoder ticks/s → joint velocity (rad/s).
pub fn ticks_s_to_rad_s(joint: &JointConfig, ticks_s: i32) -> f64 {
    let motor_rad_s = f64::from(ticks_s) / joint.encoder_ticks_per_rev * std::f64::consts::TAU;
    motor_rad_s / joint.gear_ratio * f64::from(joint.direction)
}

/// Joint torque (Nm) → drive torque command (mNm · direction / gear).
pub fn nm_to_torque_cmd(joint: &JointConfig, nm: f64) -> i32 {
    let motor_nm = nm * f64::from(joint.direction) / joint.gear_ratio;
    (motor_nm * 1000.0).round() as i32
}

pub fn torque_cmd_to_nm(joint: &JointConfig, cmd: i32) -> f64 {
    let motor_nm = f64::from(cmd) / 1000.0;
    motor_nm * joint.gear_ratio * f64::from(joint.direction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ethercat_joint::config::{JointMode, PdoOffsets};

    fn joint() -> JointConfig {
        JointConfig {
            name: "j1".into(),
            station_address: 1001,
            mode: JointMode::Csp,
            encoder_ticks_per_rev: 1000.0,
            gear_ratio: 1.0,
            direction: 1,
            position_offset_rad: 0.0,
            pdo: PdoOffsets::default(),
        }
    }

    #[test]
    fn roundtrip_position() {
        let j = joint();
        let ticks = rad_to_ticks(&j, std::f64::consts::PI);
        let back = ticks_to_rad(&j, ticks);
        assert!((back - std::f64::consts::PI).abs() < 1e-3);
    }
}
