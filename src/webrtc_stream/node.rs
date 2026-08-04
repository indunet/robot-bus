//! Wire robot-bus Node: subscribe media topics → WHEP hub.

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::config::WebrtcConfig;
use super::hub::MediaHub;
use super::media::{AudioPipeline, VideoPipeline};
use super::whep;
use crate::foxglove_msgs::msg::v1::RawAudio;
use crate::sensor_msgs::msg::v1::Image;
use crate::{Node, NodeOptions};

struct SharedVideo {
    inner: Mutex<VideoPipeline>,
}

// Safety: all encoder use is serialized by `inner`.
unsafe impl Send for SharedVideo {}
unsafe impl Sync for SharedVideo {}

struct SharedAudio {
    inner: Mutex<AudioPipeline>,
}

// Safety: all encoder use is serialized by `inner`.
unsafe impl Send for SharedAudio {}
unsafe impl Sync for SharedAudio {}

/// Build the node, start WHEP, subscribe, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = WebrtcConfig::load(&mut node, params_path)?;

    let enable_video = !cfg.image_topic.is_empty();
    let enable_audio = !cfg.audio_topic.is_empty();

    log::info!(
        "webrtc node ready: listen={} video={:?} audio={:?} data={:?}",
        cfg.listen,
        if enable_video {
            Some(&cfg.image_topic)
        } else {
            None
        },
        if enable_audio {
            Some(&cfg.audio_topic)
        } else {
            None
        },
        cfg.data_topics
    );

    let hub = MediaHub::new(
        enable_video,
        enable_audio,
        cfg.data_topics.clone(),
        cfg.fps as u32,
    );

    let _whep_thread = whep::spawn_whep_server(hub.clone(), cfg.listen)?;
    // Give the WHEP server a moment to bind before logging readiness.
    std::thread::sleep(Duration::from_millis(50));

    if enable_video {
        let video = Arc::new(SharedVideo {
            inner: Mutex::new(VideoPipeline::new(&cfg)?),
        });
        let hub_v = hub.clone();
        let topic = cfg.image_topic.clone();
        node.create_subscription::<Image, _>(
            &topic,
            move |_topic, image| {
                if let Err(e) = handle_image(&video, &hub_v, &image) {
                    log::warn!("video encode failed: {e:#}");
                }
            },
            None,
        )
        .context("create Image subscription")?;
    }

    if enable_audio {
        let audio = Arc::new(SharedAudio {
            inner: Mutex::new(AudioPipeline::new(&cfg)?),
        });
        let hub_a = hub.clone();
        let topic = cfg.audio_topic.clone();
        node.create_subscription::<RawAudio, _>(
            &topic,
            move |_topic, msg| {
                if let Err(e) = handle_audio(&audio, &hub_a, &msg) {
                    log::warn!("audio encode failed: {e:#}");
                }
            },
            None,
        )
        .context("create RawAudio subscription")?;
    }

    for topic in cfg.data_topics.clone() {
        let hub_d = hub.clone();
        let topic_name = topic.clone();
        node.create_subscription_raw(
            &topic,
            Arc::new(move |_t, payload| {
                hub_d.publish_data(&topic_name, payload.to_vec());
            }),
            None,
        )
        .with_context(|| format!("create raw subscription for {topic}"))?;
    }

    node.spin().context("node spin")?;
    Ok(())
}

fn handle_image(video: &SharedVideo, hub: &MediaHub, image: &Image) -> Result<()> {
    let mut enc = video.inner.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((data, dur)) = enc.encode(image)? {
        hub.publish_video(data, dur);
    }
    Ok(())
}

fn handle_audio(audio: &SharedAudio, hub: &MediaHub, msg: &RawAudio) -> Result<()> {
    let mut enc = audio.inner.lock().unwrap_or_else(|e| e.into_inner());
    for (data, dur) in enc.push(msg)? {
        hub.publish_audio(data, dur);
    }
    Ok(())
}
