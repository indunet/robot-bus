//! Convert between glam transforms and geometry_msgs / tf2_msgs.

use super::math::RigidTransform;
use crate::builtin_interfaces::msg::v1::Time;
use crate::geometry_msgs::msg::v1::{Quaternion, Transform, TransformStamped, Vector3};
use crate::std_msgs::msg::v1::Header;
use crate::tf2_msgs::msg::v1::TfMessage;
use glam::DQuat;

/// Stamp used for static TF (ROS convention: zero time).
pub fn static_stamp() -> Time {
    Time {
        sec: 0,
        nanosec: 0,
    }
}

/// Wall-clock stamp as ROS `Time`.
pub fn now_stamp() -> Time {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Time {
        sec: dur.as_secs() as i32,
        nanosec: dur.subsec_nanos(),
    }
}

pub fn rigid_to_msg(t: RigidTransform) -> Transform {
    Transform {
        translation: Some(Vector3 {
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        }),
        rotation: Some(Quaternion {
            x: t.rotation.x,
            y: t.rotation.y,
            z: t.rotation.z,
            w: t.rotation.w,
        }),
    }
}

pub fn msg_to_rigid(t: &Transform) -> Option<RigidTransform> {
    let tr = t.translation.as_ref()?;
    let rot = t.rotation.as_ref()?;
    Some(RigidTransform::from_xyz_xyzw(
        tr.x, tr.y, tr.z, rot.x, rot.y, rot.z, rot.w,
    ))
}

pub fn stamped_to_rigid(msg: &TransformStamped) -> Option<RigidTransform> {
    msg.transform.as_ref().and_then(msg_to_rigid)
}

pub fn make_transform_stamped(
    parent: impl Into<String>,
    child: impl Into<String>,
    transform: RigidTransform,
    stamp: Time,
) -> TransformStamped {
    TransformStamped {
        header: Some(Header {
            stamp: Some(stamp),
            frame_id: parent.into(),
        }),
        child_frame_id: child.into(),
        transform: Some(rigid_to_msg(transform)),
    }
}

pub fn make_tf_message(transforms: Vec<TransformStamped>) -> TfMessage {
    TfMessage { transforms }
}

pub fn quat_xyzw(q: DQuat) -> Quaternion {
    Quaternion {
        x: q.x,
        y: q.y,
        z: q.z,
        w: q.w,
    }
}
