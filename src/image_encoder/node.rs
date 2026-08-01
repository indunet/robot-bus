//! Wire robot-bus Node: Image in → CompressedVideo out.

use anyhow::{Context, Result};
use prost_types::Timestamp;
use std::sync::Mutex;

use super::config::EncoderConfig;
use super::encoder::FrameEncoder;
use crate::foxglove_msgs::msg::v1::CompressedVideo;
use crate::sensor_msgs::msg::v1::Image;
use crate::{Node, NodeOptions, TopicPublisher};

/// Mutex-guarded handle that is safe to share with subscription callbacks.
///
/// ZMQ publisher sockets and FFmpeg encoders are not `Sync`; all access goes
/// through the mutex so only one thread touches them at a time.
struct SharedPub {
    inner: Mutex<TopicPublisher<CompressedVideo>>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedPub {}
unsafe impl Sync for SharedPub {}

struct SharedEnc {
    inner: Mutex<FrameEncoder>,
}

// Safety: all encoder use is serialized by `inner`.
unsafe impl Send for SharedEnc {}
unsafe impl Sync for SharedEnc {}

/// Build the node from options, load config, subscribe/publish, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = EncoderConfig::load(&mut node, params_path)?;
    log::info!(
        "image encoder node ready: {} -> {} ({})",
        cfg.input_topic,
        cfg.output_topic,
        cfg.codec.as_format()
    );

    let publisher = SharedPub {
        inner: Mutex::new(
            node.create_publisher::<CompressedVideo>(&cfg.output_topic)
                .context("create CompressedVideo publisher")?,
        ),
    };
    let encoder = SharedEnc {
        inner: Mutex::new(FrameEncoder::new(&cfg)?),
    };
    let format = cfg.codec.as_format().to_string();
    let input_topic = cfg.input_topic.clone();

    node.create_subscription::<Image, _>(
        &input_topic,
        move |_topic, image| {
            if let Err(e) = handle_frame(&encoder, &publisher, &format, &image) {
                log::warn!("encode/publish failed: {e:#}");
            }
        },
        None,
    )
    .context("create Image subscription")?;

    node.spin().context("node spin")?;
    Ok(())
}

fn handle_frame(
    encoder: &SharedEnc,
    publisher: &SharedPub,
    format: &str,
    image: &Image,
) -> Result<()> {
    let data = {
        let mut enc = encoder
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match enc.encode_image(image)? {
            Some(d) => d,
            None => return Ok(()),
        }
    };

    let (seconds, nanos) = header_time(image);
    let frame_id = image
        .header
        .as_ref()
        .map(|h| h.frame_id.clone())
        .unwrap_or_default();

    let msg = CompressedVideo {
        timestamp: Some(Timestamp { seconds, nanos }),
        frame_id,
        data,
        format: format.to_string(),
    };

    let pub_ = publisher
        .inner
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    pub_.publish(&msg).context("publish CompressedVideo")?;
    Ok(())
}

fn header_time(image: &Image) -> (i64, i32) {
    let Some(h) = image.header.as_ref() else {
        return (0, 0);
    };
    let Some(t) = h.stamp.as_ref() else {
        return (0, 0);
    };
    (t.sec as i64, t.nanosec as i32)
}
