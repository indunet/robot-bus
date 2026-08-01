//! URDF subset → parent/child link transforms.
//!
//! Supports `fixed` / `revolute` / `continuous` / `prismatic`, plus URDF
//! `<mimic joint="…" multiplier="…" offset="…"/>` (ROS `robot_state_publisher`
//! semantics: `q = multiplier * q_master + offset`).

use anyhow::{bail, Context, Result};
use crate::tf::RigidTransform;
use glam::DVec3;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use urdf_rs::{JointType, Robot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovableKind {
    Revolute,
    Continuous,
    Prismatic,
}

/// URDF `<mimic>`: follow another joint's position.
#[derive(Debug, Clone, PartialEq)]
pub struct MimicSpec {
    pub joint: String,
    pub multiplier: f64,
    pub offset: f64,
}

impl MimicSpec {
    pub fn apply(&self, master_q: f64) -> f64 {
        self.multiplier * master_q + self.offset
    }
}

#[derive(Debug, Clone)]
pub struct JointModel {
    pub name: String,
    pub parent_link: String,
    pub child_link: String,
    pub origin: RigidTransform,
    pub axis: DVec3,
    pub kind: JointKind,
    /// Default position when missing from JointState (URDF lower or 0).
    pub default_position: f64,
    /// When set, position is derived from the master joint (mimic wins over JointState).
    pub mimic: Option<MimicSpec>,
}

#[derive(Debug, Clone)]
pub enum JointKind {
    Fixed,
    Movable(MovableKind),
}

#[derive(Debug, Clone)]
pub struct RobotModel {
    pub name: String,
    pub joints: Vec<JointModel>,
}

impl RobotModel {
    pub fn from_urdf_file(path: impl AsRef<Path>) -> Result<Self> {
        let robot = urdf_rs::read_file(path.as_ref())
            .with_context(|| format!("parse URDF {}", path.as_ref().display()))?;
        Self::from_robot(robot)
    }

    pub fn from_urdf_str(xml: &str) -> Result<Self> {
        let robot = urdf_rs::read_from_string(xml).context("parse URDF string")?;
        Self::from_robot(robot)
    }

    fn from_robot(robot: Robot) -> Result<Self> {
        let mut joints = Vec::with_capacity(robot.joints.len());
        for j in robot.joints {
            let kind = match j.joint_type {
                JointType::Fixed => JointKind::Fixed,
                JointType::Revolute => JointKind::Movable(MovableKind::Revolute),
                JointType::Continuous => JointKind::Movable(MovableKind::Continuous),
                JointType::Prismatic => JointKind::Movable(MovableKind::Prismatic),
                other => {
                    log::warn!(
                        "URDF joint {}: unsupported type {:?}, treating as fixed",
                        j.name,
                        other
                    );
                    JointKind::Fixed
                }
            };
            let xyz = j.origin.xyz;
            let rpy = j.origin.rpy;
            let origin = RigidTransform::from_xyz_rpy(
                xyz[0], xyz[1], xyz[2], rpy[0], rpy[1], rpy[2],
            );
            let ax = j.axis.xyz;
            let axis = DVec3::new(ax[0], ax[1], ax[2]);
            let default_position = match kind {
                JointKind::Fixed => 0.0,
                JointKind::Movable(_) => j.limit.lower,
            };
            if j.name.is_empty() || j.parent.link.is_empty() || j.child.link.is_empty() {
                bail!("URDF joint missing name/parent/child");
            }
            let mimic = match (&kind, j.mimic) {
                (JointKind::Fixed, Some(_)) => {
                    log::warn!(
                        "URDF joint {}: <mimic> on fixed joint is ignored",
                        j.name
                    );
                    None
                }
                (_, Some(m)) => {
                    if m.joint.is_empty() {
                        bail!("URDF joint {}: mimic joint name is empty", j.name);
                    }
                    if m.joint == j.name {
                        bail!("URDF joint {}: mimic cannot reference itself", j.name);
                    }
                    Some(MimicSpec {
                        joint: m.joint,
                        multiplier: m.multiplier.unwrap_or(1.0),
                        offset: m.offset.unwrap_or(0.0),
                    })
                }
                (_, None) => None,
            };
            joints.push(JointModel {
                name: j.name,
                parent_link: j.parent.link,
                child_link: j.child.link,
                origin,
                axis,
                kind,
                default_position,
                mimic,
            });
        }

        let model = Self {
            name: robot.name,
            joints,
        };
        model.validate_mimics()?;
        Ok(model)
    }

    fn validate_mimics(&self) -> Result<()> {
        let names: HashSet<&str> = self.joints.iter().map(|j| j.name.as_str()).collect();
        for j in &self.joints {
            let Some(m) = &j.mimic else { continue };
            if !names.contains(m.joint.as_str()) {
                bail!(
                    "URDF joint {}: mimic references unknown joint '{}'",
                    j.name,
                    m.joint
                );
            }
        }
        // Detect mimic cycles (A→B→A).
        for j in &self.joints {
            if j.mimic.is_none() {
                continue;
            }
            let mut seen = HashSet::new();
            let mut cur = j.name.as_str();
            while let Some(m) = self.joint_by_name(cur).and_then(|x| x.mimic.as_ref()) {
                if !seen.insert(cur) {
                    bail!("URDF mimic cycle involving joint '{}'", j.name);
                }
                cur = m.joint.as_str();
            }
        }
        Ok(())
    }

    pub fn joint_by_name(&self, name: &str) -> Option<&JointModel> {
        self.joints.iter().find(|j| j.name == name)
    }

    pub fn fixed_joints(&self) -> impl Iterator<Item = &JointModel> {
        self.joints
            .iter()
            .filter(|j| matches!(j.kind, JointKind::Fixed))
    }

    pub fn movable_joints(&self) -> impl Iterator<Item = &JointModel> {
        self.joints
            .iter()
            .filter(|j| matches!(j.kind, JointKind::Movable(_)))
    }

    /// Compute parent→child transform for a joint at position `q`.
    pub fn joint_transform(joint: &JointModel, q: f64) -> RigidTransform {
        let motion = match joint.kind {
            JointKind::Fixed => RigidTransform::identity(),
            JointKind::Movable(MovableKind::Revolute)
            | JointKind::Movable(MovableKind::Continuous) => {
                RigidTransform::from_axis_angle(joint.axis, q)
            }
            JointKind::Movable(MovableKind::Prismatic) => {
                RigidTransform::from_axis_translation(joint.axis, q)
            }
        };
        joint.origin.compose(motion)
    }

    /// Resolve positions for all movable joints from a name→position map.
    ///
    /// Mimic joints use `q = multiplier * q_master + offset` and **override**
    /// any value published for the mimic joint itself in `JointState`.
    ///
    /// The third tuple field is `true` when the effective position fell back to
    /// a default (master missing, or non-mimic joint missing).
    pub fn resolve_positions(
        &self,
        positions: &HashMap<String, f64>,
        missing_override: Option<f64>,
    ) -> Vec<(JointModel, f64, bool)> {
        let mut resolved: HashMap<String, (f64, bool)> = HashMap::new();

        // Pass 1: non-mimic movable joints from JointState / default.
        for j in self.movable_joints() {
            if j.mimic.is_some() {
                continue;
            }
            if let Some(&q) = positions.get(&j.name) {
                resolved.insert(j.name.clone(), (q, false));
            } else {
                let q = missing_override.unwrap_or(j.default_position);
                resolved.insert(j.name.clone(), (q, true));
            }
        }

        // Pass 2+: mimic joints (supports chains master→a→b).
        let mimic_joints: Vec<&JointModel> = self
            .movable_joints()
            .filter(|j| j.mimic.is_some())
            .collect();
        let mut pending: Vec<&JointModel> = mimic_joints;
        let mut guard = 0;
        while !pending.is_empty() && guard < self.joints.len() + 1 {
            guard += 1;
            let before = pending.len();
            let mut next = Vec::new();
            for j in pending {
                let m = j.mimic.as_ref().expect("filtered");
                if let Some(&(master_q, master_missing)) = resolved.get(&m.joint) {
                    let q = m.apply(master_q);
                    resolved.insert(j.name.clone(), (q, master_missing));
                } else if let Some(master) = self.joint_by_name(&m.joint) {
                    // Master not yet resolved: if master is fixed, treat as 0;
                    // if master is another mimic still pending, defer.
                    if matches!(master.kind, JointKind::Fixed) {
                        let q = m.apply(0.0);
                        resolved.insert(j.name.clone(), (q, false));
                    } else if master.mimic.is_some() {
                        next.push(j);
                    } else {
                        // Movable master missing from resolved (shouldn't happen)
                        let master_q = missing_override.unwrap_or(master.default_position);
                        let q = m.apply(master_q);
                        resolved.insert(j.name.clone(), (q, true));
                    }
                } else {
                    // Unknown master — validated at load; treat as default.
                    let q = missing_override.unwrap_or(j.default_position);
                    resolved.insert(j.name.clone(), (q, true));
                }
            }
            if next.len() == before {
                // Unresolvable remainder (should only hit if validation missed a cycle).
                for j in next {
                    let q = missing_override.unwrap_or(j.default_position);
                    resolved.insert(j.name.clone(), (q, true));
                }
                break;
            }
            pending = next;
        }

        self.movable_joints()
            .map(|j| {
                let (q, missing) = resolved
                    .get(&j.name)
                    .copied()
                    .unwrap_or((missing_override.unwrap_or(j.default_position), true));
                (j.clone(), q, missing)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robot_state_publisher::EXAMPLE_URDF;

    #[test]
    fn parse_simple_arm() {
        let model = RobotModel::from_urdf_str(EXAMPLE_URDF).unwrap();
        assert_eq!(model.movable_joints().count(), 2);
        assert_eq!(model.fixed_joints().count(), 1);

        let j1 = model.joints.iter().find(|j| j.name == "joint_1").unwrap();
        let t0 = RobotModel::joint_transform(j1, 0.0);
        assert!((t0.translation.z - 0.1).abs() < 1e-9);

        let t90 = RobotModel::joint_transform(j1, std::f64::consts::FRAC_PI_2);
        let p = t90.transform_point(glam::DVec3::new(1.0, 0.0, 0.0));
        // origin z=0.1, rotate 90° about Z: (1,0,0) → (0,1,0) then +origin
        assert!((p.x).abs() < 1e-9);
        assert!((p.y - 1.0).abs() < 1e-9);
        assert!((p.z - 0.1).abs() < 1e-9);
    }

    const MIMIC_URDF: &str = r#"<?xml version="1.0"?>
<robot name="gripper">
  <link name="base"/>
  <link name="left"/>
  <link name="right"/>
  <joint name="finger_left" type="prismatic">
    <parent link="base"/>
    <child link="left"/>
    <origin xyz="0 0.05 0" rpy="0 0 0"/>
    <axis xyz="0 1 0"/>
    <limit lower="0" upper="0.05" effort="10" velocity="1"/>
  </joint>
  <joint name="finger_right" type="prismatic">
    <parent link="base"/>
    <child link="right"/>
    <origin xyz="0 -0.05 0" rpy="0 0 0"/>
    <axis xyz="0 1 0"/>
    <limit lower="-0.05" upper="0" effort="10" velocity="1"/>
    <mimic joint="finger_left" multiplier="-1.0" offset="0.0"/>
  </joint>
</robot>
"#;

    #[test]
    fn mimic_follows_master() {
        let model = RobotModel::from_urdf_str(MIMIC_URDF).unwrap();
        let right = model.joint_by_name("finger_right").unwrap();
        assert_eq!(
            right.mimic.as_ref().unwrap().joint,
            "finger_left"
        );

        let mut positions = HashMap::new();
        positions.insert("finger_left".into(), 0.02);
        // Explicit wrong value for mimic joint must be ignored.
        positions.insert("finger_right".into(), 9.0);

        let resolved = model.resolve_positions(&positions, None);
        let left_q = resolved
            .iter()
            .find(|(j, _, _)| j.name == "finger_left")
            .unwrap()
            .1;
        let (right_j, right_q, missing) = resolved
            .iter()
            .find(|(j, _, _)| j.name == "finger_right")
            .unwrap();
        assert!((left_q - 0.02).abs() < 1e-12);
        assert!((right_q - (-0.02)).abs() < 1e-12);
        assert!(!missing);
        assert!(right_j.mimic.is_some());
    }

    #[test]
    fn mimic_cycle_rejected() {
        let xml = r#"<?xml version="1.0"?>
<robot name="bad">
  <link name="a"/><link name="b"/>
  <joint name="j1" type="revolute">
    <parent link="a"/><child link="b"/>
    <axis xyz="0 0 1"/>
    <limit lower="0" upper="1" effort="1" velocity="1"/>
    <mimic joint="j2"/>
  </joint>
  <joint name="j2" type="revolute">
    <parent link="a"/><child link="b"/>
    <axis xyz="0 0 1"/>
    <limit lower="0" upper="1" effort="1" velocity="1"/>
    <mimic joint="j1"/>
  </joint>
</robot>"#;
        assert!(RobotModel::from_urdf_str(xml).is_err());
    }
}
