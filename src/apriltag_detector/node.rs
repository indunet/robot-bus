//! Wire robot-bus Node: Image in → AprilTagDetectionArray out.

use anyhow::{Context, Result};
use std::sync::Mutex;

use super::config::DetectorConfig;
use super::detector::TagDetector;
use crate::apriltag_msgs::msg::v1::AprilTagDetectionArray;
use crate::sensor_msgs::msg::v1::Image;
use crate::{Node, NodeOptions, TopicPublisher};

/// Mutex-guarded publisher safe to share with subscription callbacks.
struct SharedPub {
    inner: Mutex<TopicPublisher<AprilTagDetectionArray>>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedPub {}
unsafe impl Sync for SharedPub {}

struct SharedDet {
    inner: Mutex<TagDetector>,
}

// Safety: all detector use is serialized by `inner`.
unsafe impl Send for SharedDet {}
unsafe impl Sync for SharedDet {}

/// Build the node from options, load config, subscribe/publish, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = DetectorConfig::load(&mut node, params_path)?;
    log::info!(
        "apriltag detector ready: {} -> {} (family={}, decimate={})",
        cfg.input_topic,
        cfg.output_topic,
        cfg.family,
        cfg.decimate
    );

    let publisher = SharedPub {
        inner: Mutex::new(
            node.create_publisher::<AprilTagDetectionArray>(&cfg.output_topic)
                .context("create AprilTagDetectionArray publisher")?,
        ),
    };
    let detector = SharedDet {
        inner: Mutex::new(TagDetector::new(&cfg)?),
    };
    let input_topic = cfg.input_topic.clone();

    node.create_subscription::<Image, _>(
        &input_topic,
        move |_topic, image| {
            if let Err(e) = handle_frame(&detector, &publisher, &image) {
                log::warn!("detect/publish failed: {e:#}");
            }
        },
        None,
    )
    .context("create Image subscription")?;

    node.spin().context("node spin")?;
    Ok(())
}

fn handle_frame(detector: &SharedDet, publisher: &SharedPub, image: &Image) -> Result<()> {
    let msg = {
        let mut det = detector.inner.lock().unwrap_or_else(|e| e.into_inner());
        det.detect_array(image)?
    };

    if !msg.detections.is_empty() {
        log::debug!(
            "detected {} tag(s): {:?}",
            msg.detections.len(),
            msg.detections
                .iter()
                .map(|d| d.id)
                .collect::<Vec<_>>()
        );
    }

    let pub_ = publisher.inner.lock().unwrap_or_else(|e| e.into_inner());
    pub_
        .publish(&msg)
        .context("publish AprilTagDetectionArray")?;
    Ok(())
}
