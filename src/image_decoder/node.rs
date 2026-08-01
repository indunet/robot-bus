//! Wire robot-bus Node: CompressedVideo in → Image out.

use anyhow::{Context, Result};
use std::sync::Mutex;

use super::config::DecoderConfig;
use super::decoder::FrameDecoder;
use crate::builtin_interfaces::msg::v1::Time;
use crate::foxglove_msgs::msg::v1::CompressedVideo;
use crate::sensor_msgs::msg::v1::Image;
use crate::{Node, NodeOptions, TopicPublisher};

/// Mutex-guarded handle that is safe to share with subscription callbacks.
struct SharedPub {
    inner: Mutex<TopicPublisher<Image>>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedPub {}
unsafe impl Sync for SharedPub {}

struct SharedDec {
    inner: Mutex<FrameDecoder>,
}

// Safety: all decoder use is serialized by `inner`.
unsafe impl Send for SharedDec {}
unsafe impl Sync for SharedDec {}

/// Build the node from options, load config, subscribe/publish, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = DecoderConfig::load(&mut node, params_path)?;
    log::info!(
        "image decoder node ready: {} -> {} (fallback codec {}, output {})",
        cfg.input_topic,
        cfg.output_topic,
        cfg.codec.as_format(),
        cfg.output_encoding.as_str()
    );

    let publisher = SharedPub {
        inner: Mutex::new(
            node.create_publisher::<Image>(&cfg.output_topic)
                .context("create Image publisher")?,
        ),
    };
    let decoder = SharedDec {
        inner: Mutex::new(FrameDecoder::new(&cfg)?),
    };
    let input_topic = cfg.input_topic.clone();

    node.create_subscription::<CompressedVideo, _>(
        &input_topic,
        move |_topic, video| {
            if let Err(e) = handle_frame(&decoder, &publisher, &video) {
                log::warn!("decode/publish failed: {e:#}");
            }
        },
        None,
    )
    .context("create CompressedVideo subscription")?;

    node.spin().context("node spin")?;
    Ok(())
}

fn handle_frame(
    decoder: &SharedDec,
    publisher: &SharedPub,
    video: &CompressedVideo,
) -> Result<()> {
    let stamp = video.timestamp.as_ref().map(|t| Time {
        sec: t.seconds as i32,
        nanosec: t.nanos as u32,
    });

    let image = {
        let mut dec = decoder.inner.lock().unwrap_or_else(|e| e.into_inner());
        match dec.decode_access_unit(&video.data, &video.format, &video.frame_id, stamp)? {
            Some(img) => img,
            None => return Ok(()),
        }
    };

    let pub_ = publisher.inner.lock().unwrap_or_else(|e| e.into_inner());
    pub_.publish(&image).context("publish Image")?;
    Ok(())
}
