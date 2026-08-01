//! SE(3) transform math (glam f64).

use glam::{DAffine3, DMat3, DQuat, DVec3, EulerRot};

/// Rigid transform: point in child → point in parent (`p_parent = T * p_child`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidTransform {
    pub translation: DVec3,
    pub rotation: DQuat,
}

impl Default for RigidTransform {
    fn default() -> Self {
        Self::identity()
    }
}

impl RigidTransform {
    pub fn identity() -> Self {
        Self {
            translation: DVec3::ZERO,
            rotation: DQuat::IDENTITY,
        }
    }

    pub fn from_translation_rotation(translation: DVec3, rotation: DQuat) -> Self {
        Self {
            translation,
            rotation: rotation.normalize(),
        }
    }

    /// URDF / ROS RPY: `R = Rz(yaw) * Ry(pitch) * Rx(roll)`.
    pub fn from_xyz_rpy(x: f64, y: f64, z: f64, roll: f64, pitch: f64, yaw: f64) -> Self {
        let rotation = DQuat::from_euler(EulerRot::ZYX, yaw, pitch, roll);
        Self::from_translation_rotation(DVec3::new(x, y, z), rotation)
    }

    pub fn from_xyz_xyzw(x: f64, y: f64, z: f64, qx: f64, qy: f64, qz: f64, qw: f64) -> Self {
        Self::from_translation_rotation(DVec3::new(x, y, z), DQuat::from_xyzw(qx, qy, qz, qw))
    }

    pub fn inverse(self) -> Self {
        let rotation = self.rotation.conjugate();
        let translation = rotation * (-self.translation);
        Self {
            translation,
            rotation,
        }
    }

    /// Compose `self * other`: apply `other` first, then `self`
    /// (`T_a_c = T_a_b * T_b_c`).
    pub fn compose(self, other: Self) -> Self {
        Self {
            translation: self.translation + self.rotation * other.translation,
            rotation: (self.rotation * other.rotation).normalize(),
        }
    }

    pub fn transform_point(self, p: DVec3) -> DVec3 {
        self.translation + self.rotation * p
    }

    pub fn transform_vector(self, v: DVec3) -> DVec3 {
        self.rotation * v
    }

    pub fn to_affine(self) -> DAffine3 {
        DAffine3::from_rotation_translation(self.rotation, self.translation)
    }

    /// Rotation about a (possibly non-unit) axis by `angle` radians.
    pub fn from_axis_angle(axis: DVec3, angle: f64) -> Self {
        let axis = if axis.length_squared() < 1e-18 {
            DVec3::X
        } else {
            axis.normalize()
        };
        Self::from_translation_rotation(DVec3::ZERO, DQuat::from_axis_angle(axis, angle))
    }

    /// Translation along a (possibly non-unit) axis by `distance` metres.
    pub fn from_axis_translation(axis: DVec3, distance: f64) -> Self {
        let axis = if axis.length_squared() < 1e-18 {
            DVec3::X
        } else {
            axis.normalize()
        };
        Self::from_translation_rotation(axis * distance, DQuat::IDENTITY)
    }

    /// Rotation matrix columns (for debug / tests).
    pub fn rotation_matrix(self) -> DMat3 {
        DMat3::from_quat(self.rotation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_inverse_identity() {
        let t = RigidTransform::from_xyz_rpy(1.0, 2.0, 3.0, 0.1, -0.2, 0.3);
        let id = t.compose(t.inverse());
        assert!(id.translation.length() < 1e-9);
        assert!((id.rotation.w.abs() - 1.0).abs() < 1e-9 || id.rotation.length() > 0.99);
        let p = DVec3::new(0.5, -1.0, 2.0);
        let back = t.inverse().transform_point(t.transform_point(p));
        assert!((back - p).length() < 1e-9);
    }

    #[test]
    fn chain_a_b_c() {
        let a_b = RigidTransform::from_xyz_rpy(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let b_c = RigidTransform::from_xyz_rpy(0.0, 2.0, 0.0, 0.0, 0.0, std::f64::consts::FRAC_PI_2);
        let a_c = a_b.compose(b_c);
        let p_c = DVec3::new(1.0, 0.0, 0.0);
        let p_a = a_c.transform_point(p_c);
        // In B: Rz(90°)*(1,0,0)+(0,2,0)=(0,1,0)+(0,2,0)=(0,3,0); in A: +(1,0,0) → (1,3,0)
        assert!((p_a - DVec3::new(1.0, 3.0, 0.0)).length() < 1e-9);
    }
}
