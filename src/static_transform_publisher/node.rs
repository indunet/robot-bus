//! Wire robot-bus Node: YAML static transforms → `/tf_static`.

use super::config::StaticTransformConfig;
use anyhow::{Context, Result};
use crate::tf::{
    make_transform_stamped, static_stamp, TransformBroadcaster,
};
use crate::{Node, NodeOptions, TimerCallback, TopicPublisher};
use crate::tf2_msgs::msg::v1::TfMessage;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct SharedBroadcaster {
    inner: Mutex<TransformBroadcaster>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedBroadcaster {}
unsafe impl Sync for SharedBroadcaster {}

/// Build the node, publish static transforms, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = StaticTransformConfig::load(&mut node, params_path)?;

    let publisher: TopicPublisher<TfMessage> = node
        .create_publisher::<TfMessage>(&cfg.output_topic)
        .context("create TFMessage publisher")?;
    let broadcaster = Arc::new(SharedBroadcaster {
        inner: Mutex::new(TransformBroadcaster::new(publisher)),
    });

    let stamped: Vec<_> = cfg
        .transforms
        .iter()
        .map(|entry| {
            let rigid = entry.to_rigid()?;
            Ok(make_transform_stamped(
                entry.parent_frame_id.clone(),
                entry.child_frame_id.clone(),
                rigid,
                static_stamp(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    let publish = {
        let bc = Arc::clone(&broadcaster);
        let transforms = stamped.clone();
        move || {
            if let Ok(guard) = bc.inner.lock() {
                if let Err(e) = guard.send_transforms(transforms.clone()) {
                    log::warn!("publish static TF failed: {e}");
                }
            }
        }
    };

    publish();
    log::info!(
        "static transform publisher ready: {} transform(s) → {} @ {} Hz",
        cfg.transforms.len(),
        cfg.output_topic,
        cfg.publish_rate_hz
    );

    if cfg.publish_rate_hz > 0.0 {
        let period = Duration::from_secs_f64(1.0 / cfg.publish_rate_hz);
        let cb: TimerCallback = Arc::new(publish);
        node.create_timer(period, cb, None)
            .context("create static TF republish timer")?;
    }

    node.spin().context("spin static transform publisher")?;
    Ok(())
}
