//! Wire robot-bus Node: URDF + JointState → `/tf` + `/tf_static`.

use super::config::RobotStatePublisherConfig;
use super::kinematics::RobotModel;
use anyhow::{Context, Result};
use crate::geometry_msgs::msg::v1::TransformStamped;
use crate::sensor_msgs::msg::v1::JointState;
use crate::tf::{
    make_transform_stamped, now_stamp, static_stamp, TransformBroadcaster,
};
use crate::tf2_msgs::msg::v1::TfMessage;
use crate::{Node, NodeOptions, TimerCallback, TopicPublisher};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct SharedBroadcaster {
    inner: Mutex<TransformBroadcaster>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedBroadcaster {}
unsafe impl Sync for SharedBroadcaster {}

/// Build the node, publish TF from URDF + joint states, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = RobotStatePublisherConfig::load(&mut node, params_path)?;
    let model = RobotModel::from_urdf_file(cfg.urdf_path())?;

    let tf_pub: TopicPublisher<TfMessage> = node
        .create_publisher::<TfMessage>(&cfg.tf_topic)
        .context("create /tf publisher")?;
    let tf_static_pub: TopicPublisher<TfMessage> = node
        .create_publisher::<TfMessage>(&cfg.tf_static_topic)
        .context("create /tf_static publisher")?;

    let dynamic_bc = Arc::new(SharedBroadcaster {
        inner: Mutex::new(TransformBroadcaster::new(tf_pub)),
    });
    let static_bc = Arc::new(SharedBroadcaster {
        inner: Mutex::new(TransformBroadcaster::new(tf_static_pub)),
    });

    let static_stamped: Vec<TransformStamped> = model
        .fixed_joints()
        .map(|j| {
            make_transform_stamped(
                j.parent_link.clone(),
                j.child_link.clone(),
                RobotModel::joint_transform(j, 0.0),
                static_stamp(),
            )
        })
        .collect();

    let publish_static = {
        let bc = Arc::clone(&static_bc);
        let transforms = static_stamped.clone();
        move || {
            if transforms.is_empty() {
                return;
            }
            if let Ok(guard) = bc.inner.lock() {
                if let Err(e) = guard.send_transforms(transforms.clone()) {
                    log::warn!("publish fixed TF failed: {e}");
                }
            }
        }
    };
    publish_static();

    if cfg.static_publish_rate_hz > 0.0 && !static_stamped.is_empty() {
        let period = Duration::from_secs_f64(1.0 / cfg.static_publish_rate_hz);
        let cb: TimerCallback = Arc::new(publish_static);
        node.create_timer(period, cb, None)
            .context("create fixed TF republish timer")?;
    }

    let model = Arc::new(model);
    let missing = cfg.missing_joint_position;
    let dyn_bc = Arc::clone(&dynamic_bc);
    let model_cb = Arc::clone(&model);
    let warned_missing = Arc::new(Mutex::new(HashSet::<String>::new()));
    node.create_subscription::<JointState, _>(
        &cfg.joint_states_topic,
        move |_topic, msg| {
            let mut positions = HashMap::new();
            for (i, name) in msg.name.iter().enumerate() {
                if let Some(q) = msg.position.get(i) {
                    positions.insert(name.clone(), *q);
                }
            }
            let resolved = model_cb.resolve_positions(&positions, missing);
            if let Ok(mut warned) = warned_missing.lock() {
                for (joint, _q, was_missing) in &resolved {
                    if *was_missing && warned.insert(joint.name.clone()) {
                        if joint.mimic.is_some() {
                            log::warn!(
                                "mimic joint {} (or its master) missing from JointState; using default",
                                joint.name
                            );
                        } else {
                            log::warn!(
                                "joint {} missing from JointState; using default position",
                                joint.name
                            );
                        }
                    }
                }
            }
            let stamp = msg
                .header
                .as_ref()
                .and_then(|h| h.stamp)
                .unwrap_or_else(now_stamp);
            let transforms: Vec<_> = resolved
                .into_iter()
                .map(|(joint, q, _)| {
                    let transform = RobotModel::joint_transform(&joint, q);
                    make_transform_stamped(
                        joint.parent_link,
                        joint.child_link,
                        transform,
                        stamp,
                    )
                })
                .collect();
            if transforms.is_empty() {
                return;
            }
            if let Ok(guard) = dyn_bc.inner.lock() {
                if let Err(e) = guard.send_transforms(transforms) {
                    log::warn!("publish dynamic TF failed: {e}");
                }
            }
        },
        None,
    )
    .context("subscribe JointState")?;

    log::info!(
        "robot_state_publisher ready: urdf={} movable={} fixed={} ({} → {}, {} → {})",
        cfg.urdf_file.display(),
        model.movable_joints().count(),
        model.fixed_joints().count(),
        cfg.joint_states_topic,
        cfg.tf_topic,
        "fixed",
        cfg.tf_static_topic
    );

    node.spin().context("spin robot_state_publisher")?;
    Ok(())
}
