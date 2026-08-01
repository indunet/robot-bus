//! In-memory TF tree buffer (static + latest dynamic edges).

use super::convert::{make_transform_stamped, msg_to_rigid, static_stamp};
use super::error::TfError;
use super::math::RigidTransform;
use crate::builtin_interfaces::msg::v1::Time;
use crate::geometry_msgs::msg::v1::TransformStamped;
use crate::tf2_msgs::msg::v1::TfMessage;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
struct Edge {
    parent: String,
    child: String,
    transform: RigidTransform,
    stamp: Time,
    is_static: bool,
}

/// Stores parent→child transforms and answers `lookup_transform`.
///
/// Time semantics (v1):
/// - Static edges ignore query time and always apply.
/// - Dynamic edges use the **latest** sample (no interpolation / extrapolation).
#[derive(Debug, Default, Clone)]
pub struct Buffer {
    /// Keyed by child frame (one parent per child, last write wins).
    by_child: HashMap<String, Edge>,
}

impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.by_child.clear();
    }

    /// Ingest a ROS `TFMessage`. `is_static` marks `/tf_static` traffic.
    pub fn set_transform_msg(&mut self, msg: &TfMessage, is_static: bool) {
        for stamped in &msg.transforms {
            let _ = self.set_transform_stamped(stamped, is_static);
        }
    }

    pub fn set_transform_stamped(
        &mut self,
        stamped: &TransformStamped,
        is_static: bool,
    ) -> Result<(), TfError> {
        let parent = stamped
            .header
            .as_ref()
            .map(|h| h.frame_id.as_str())
            .unwrap_or("")
            .to_string();
        let child = stamped.child_frame_id.clone();
        if parent.is_empty() || child.is_empty() {
            return Err(TfError::Invalid(
                "parent and child frame_id must be non-empty".into(),
            ));
        }
        if parent == child {
            return Err(TfError::Invalid(format!(
                "parent and child must differ ({parent})"
            )));
        }
        let transform = stamped
            .transform
            .as_ref()
            .and_then(msg_to_rigid)
            .ok_or_else(|| TfError::Invalid("missing transform payload".into()))?;
        let stamp = stamped
            .header
            .as_ref()
            .and_then(|h| h.stamp)
            .unwrap_or_else(static_stamp);

        // Static edges win over dynamic for the same child; do not let dynamic overwrite static.
        if let Some(existing) = self.by_child.get(&child) {
            if existing.is_static && !is_static {
                return Ok(());
            }
        }

        self.by_child.insert(
            child.clone(),
            Edge {
                parent,
                child,
                transform,
                stamp,
                is_static,
            },
        );
        Ok(())
    }

    pub fn set_transform(
        &mut self,
        parent: impl Into<String>,
        child: impl Into<String>,
        transform: RigidTransform,
        stamp: Time,
        is_static: bool,
    ) -> Result<(), TfError> {
        let stamped = make_transform_stamped(parent, child, transform, stamp);
        self.set_transform_stamped(&stamped, is_static)
    }

    pub fn can_transform(&self, target: &str, source: &str) -> bool {
        self.lookup_transform(target, source, None).is_ok()
    }

    /// Return transform of `source` relative to `target`
    /// (`header.frame_id = target`, `child_frame_id = source`).
    ///
    /// `time` is currently ignored for dynamic edges (latest only); reserved for future use.
    pub fn lookup_transform(
        &self,
        target: &str,
        source: &str,
        _time: Option<Time>,
    ) -> Result<TransformStamped, TfError> {
        if target.is_empty() || source.is_empty() {
            return Err(TfError::Invalid(
                "target and source frame must be non-empty".into(),
            ));
        }
        if target == source {
            return Ok(make_transform_stamped(
                target,
                source,
                RigidTransform::identity(),
                static_stamp(),
            ));
        }

        // For each visited frame F, store T_F_source (transform of source in F).
        let mut from_source: HashMap<String, RigidTransform> = HashMap::new();
        from_source.insert(source.to_string(), RigidTransform::identity());
        let mut queue = VecDeque::new();
        queue.push_back(source.to_string());

        // Adjacency: undirected walk using child→parent map.
        let neighbors = |frame: &str| -> Vec<(String, RigidTransform, bool)> {
            let mut out = Vec::new();
            // Up: child → parent  (T_parent_source = T_parent_child * T_child_source)
            if let Some(edge) = self.by_child.get(frame) {
                out.push((edge.parent.clone(), edge.transform, true));
            }
            // Down: parent → children
            for edge in self.by_child.values() {
                if edge.parent == frame {
                    out.push((edge.child.clone(), edge.transform, false));
                }
            }
            out
        };

        let mut stamp = static_stamp();
        while let Some(frame) = queue.pop_front() {
            if frame == target {
                let t = from_source.get(&frame).copied().unwrap_or_default();
                return Ok(make_transform_stamped(target, source, t, stamp));
            }
            let t_frame_source = match from_source.get(&frame).copied() {
                Some(t) => t,
                None => continue,
            };
            for (next, edge_t, going_up) in neighbors(&frame) {
                if from_source.contains_key(&next) {
                    continue;
                }
                let t_next_source = if going_up {
                    // frame is child, next is parent
                    edge_t.compose(t_frame_source)
                } else {
                    // frame is parent, next is child: T_child_source = inv(T_parent_child) * T_parent_source
                    edge_t.inverse().compose(t_frame_source)
                };
                // Prefer a non-zero stamp from the path when available.
                if let Some(edge) = self.by_child.get(if going_up { &frame } else { &next }) {
                    if !edge.is_static && (edge.stamp.sec != 0 || edge.stamp.nanosec != 0) {
                        stamp = edge.stamp;
                    }
                }
                from_source.insert(next.clone(), t_next_source);
                queue.push_back(next);
            }
        }

        // Distinguish unknown frames vs disconnected.
        let known: HashSet<&str> = self
            .by_child
            .iter()
            .flat_map(|(c, e)| [c.as_str(), e.parent.as_str()])
            .collect();
        if !known.contains(target) {
            return Err(TfError::UnknownFrame(target.to_string()));
        }
        if !known.contains(source) {
            return Err(TfError::UnknownFrame(source.to_string()));
        }
        Err(TfError::connectivity(target, source))
    }

    /// All known frame ids.
    pub fn frames(&self) -> Vec<String> {
        let mut set = HashSet::new();
        for (child, edge) in &self.by_child {
            set.insert(child.clone());
            set.insert(edge.parent.clone());
        }
        let mut out: Vec<_> = set.into_iter().collect();
        out.sort();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tf::convert::now_stamp;

    fn push(buf: &mut Buffer, parent: &str, child: &str, t: RigidTransform, is_static: bool) {
        buf.set_transform(parent, child, t, now_stamp(), is_static)
            .unwrap();
    }

    #[test]
    fn chain_lookup() {
        let mut buf = Buffer::new();
        push(
            &mut buf,
            "a",
            "b",
            RigidTransform::from_xyz_rpy(1.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            true,
        );
        push(
            &mut buf,
            "b",
            "c",
            RigidTransform::from_xyz_rpy(0.0, 2.0, 0.0, 0.0, 0.0, 0.0),
            false,
        );

        let a_c = buf.lookup_transform("a", "c", None).unwrap();
        let rigid = crate::tf::convert::stamped_to_rigid(&a_c).unwrap();
        assert!((rigid.translation.x - 1.0).abs() < 1e-9);
        assert!((rigid.translation.y - 2.0).abs() < 1e-9);

        let c_a = buf.lookup_transform("c", "a", None).unwrap();
        let inv = crate::tf::convert::stamped_to_rigid(&c_a).unwrap();
        let p = glam::DVec3::new(0.0, 0.0, 0.0);
        let back = rigid.transform_point(inv.transform_point(p));
        assert!(back.length() < 1e-9);

        assert!(buf.can_transform("a", "c"));
        assert!(!buf.can_transform("a", "missing"));
    }

    #[test]
    fn static_blocks_dynamic_overwrite() {
        let mut buf = Buffer::new();
        push(
            &mut buf,
            "base",
            "cam",
            RigidTransform::from_xyz_rpy(1.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            true,
        );
        push(
            &mut buf,
            "base",
            "cam",
            RigidTransform::from_xyz_rpy(9.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            false,
        );
        let t = buf.lookup_transform("base", "cam", None).unwrap();
        let rigid = crate::tf::convert::stamped_to_rigid(&t).unwrap();
        assert!((rigid.translation.x - 1.0).abs() < 1e-9);
    }

    #[test]
    fn identity_same_frame() {
        let buf = Buffer::new();
        let t = buf.lookup_transform("x", "x", None).unwrap();
        assert_eq!(t.child_frame_id, "x");
    }
}
