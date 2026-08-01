//! Wire robot-bus Node: USB camera → `sensor_msgs/Image` (`rgb8`) out.

use super::config::CaptureConfig;
use super::device::{describe_index, resolve_camera_index};
use super::frame::rgb8_image;
use anyhow::{Context, Result};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::threaded::CallbackCamera;
use nokhwa::utils::{CameraFormat, FrameFormat, RequestedFormat, RequestedFormatType, Resolution};
use crate::sensor_msgs::msg::v1::Image;
use crate::{Node, NodeOptions, TopicPublisher};
use std::sync::{Arc, Mutex};

/// Mutex-guarded publisher shared with the nokhwa callback thread.
struct SharedPub {
    inner: Mutex<TopicPublisher<Image>>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedPub {}
unsafe impl Sync for SharedPub {}

/// Build the node, open the camera, publish frames, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = CaptureConfig::load(&mut node, params_path)?;
    let index = resolve_camera_index(&cfg.device)?;
    let label = describe_index(&index);

    let publisher = Arc::new(SharedPub {
        inner: Mutex::new(
            node.create_publisher::<Image>(&cfg.output_topic)
                .context("create Image publisher")?,
        ),
    });

    let frame_id = cfg.frame_id.clone();
    let pub_cb = Arc::clone(&publisher);
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
        CameraFormat::new(
            Resolution::new(cfg.width, cfg.height),
            FrameFormat::MJPEG,
            cfg.fps,
        ),
    ));

    let mut camera = CallbackCamera::new(index, requested, move |buffer| {
        let decoded = match buffer.decode_image::<RgbFormat>() {
            Ok(img) => img,
            Err(e) => {
                log::warn!("decode frame to rgb8 failed: {e}");
                return;
            }
        };
        let width = decoded.width();
        let height = decoded.height();
        let data = decoded.into_raw();
        let msg = rgb8_image(width, height, &frame_id, data);
        if let Err(e) = publish_image(&pub_cb, &msg) {
            log::warn!("publish Image failed: {e:#}");
        }
    })
    .context("open camera")?;

    camera.open_stream().context("open camera stream")?;

    let actual = camera
        .camera_format()
        .context("read camera format after open")?;
    log::info!(
        "usb camera ready: {label} -> {} (requested {}x{}@{} → got {}x{}@{} {:?})",
        cfg.output_topic,
        cfg.width,
        cfg.height,
        cfg.fps,
        actual.resolution().width(),
        actual.resolution().height(),
        actual.frame_rate(),
        actual.format()
    );

    // Keep CallbackCamera alive for the lifetime of spin.
    let spin_result = node.spin().context("node spin");
    if let Err(e) = camera.stop_stream() {
        log::debug!("stop camera stream: {e}");
    }
    spin_result
}

fn publish_image(publisher: &SharedPub, msg: &Image) -> Result<()> {
    let pub_ = publisher
        .inner
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    pub_.publish(msg).context("publish Image")?;
    Ok(())
}
