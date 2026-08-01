//! Wire robot-bus Node: RawAudio in → speaker out.

use super::config::PlayConfig;
use super::device::resolve_output_device;
use super::pcm::{decode_pcm_s16, i16_to_f32, max_samples};
use anyhow::{bail, Context, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig};
use crate::foxglove_msgs::msg::v1::RawAudio;
use crate::{Node, NodeOptions};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

struct PcmRing {
    samples: VecDeque<i16>,
    max_samples: usize,
}

impl PcmRing {
    fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples.min(64 * 1024)),
            max_samples: max_samples.max(1),
        }
    }

    fn push(&mut self, data: &[i16]) {
        for &s in data {
            if self.samples.len() >= self.max_samples {
                self.samples.pop_front();
            }
            self.samples.push_back(s);
        }
    }

    fn pop_into(&mut self, out: &mut [i16]) {
        for slot in out.iter_mut() {
            *slot = self.samples.pop_front().unwrap_or(0);
        }
    }
}

/// Build the node, open the output stream, subscribe, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = PlayConfig::load(&mut node, params_path)?;
    let device = resolve_output_device(&cfg.device)?;
    let device_name = device.name().unwrap_or_else(|_| "<unknown>".into());

    log::info!(
        "audio play ready: {} -> device={device_name:?} ({} Hz, {} ch, buffer {} ms)",
        cfg.input_topic,
        cfg.sample_rate,
        cfg.channels,
        cfg.max_buffer_ms
    );

    let ring = Arc::new(Mutex::new(PcmRing::new(max_samples(
        cfg.sample_rate,
        cfg.channels,
        cfg.max_buffer_ms,
    ))));

    let stream = build_output_stream(&device, &cfg, Arc::clone(&ring))?;
    stream.play().context("start output stream")?;

    let expected_rate = cfg.sample_rate;
    let expected_channels = u32::from(cfg.channels);
    let input_topic = cfg.input_topic.clone();
    let ring_sub = Arc::clone(&ring);

    node.create_subscription::<RawAudio, _>(
        &input_topic,
        move |_topic, msg| {
            if let Err(e) = enqueue_audio(&ring_sub, &msg, expected_rate, expected_channels) {
                log::warn!("drop RawAudio: {e:#}");
            }
        },
        None,
    )
    .context("create RawAudio subscription")?;

    node.spin().context("node spin")?;
    drop(stream);
    Ok(())
}

fn enqueue_audio(
    ring: &Mutex<PcmRing>,
    msg: &RawAudio,
    expected_rate: u32,
    expected_channels: u32,
) -> Result<()> {
    if msg.format != "pcm-s16" {
        bail!("unsupported format {:?}; expected pcm-s16", msg.format);
    }
    if msg.sample_rate != expected_rate {
        bail!(
            "sample_rate mismatch: msg={} expected={}",
            msg.sample_rate,
            expected_rate
        );
    }
    if msg.number_of_channels != expected_channels {
        bail!(
            "channels mismatch: msg={} expected={}",
            msg.number_of_channels,
            expected_channels
        );
    }

    let samples = decode_pcm_s16(&msg.data)?;
    if samples.len() % expected_channels as usize != 0 {
        bail!(
            "sample count {} not divisible by channels {}",
            samples.len(),
            expected_channels
        );
    }

    let mut ring = ring.lock().unwrap_or_else(|e| e.into_inner());
    ring.push(&samples);
    Ok(())
}

fn build_output_stream(
    device: &cpal::Device,
    cfg: &PlayConfig,
    ring: Arc<Mutex<PcmRing>>,
) -> Result<cpal::Stream> {
    let supported = device
        .default_output_config()
        .context("default output config")?;

    let sample_format = supported.sample_format();
    let stream_config = StreamConfig {
        channels: cfg.channels,
        sample_rate: cpal::SampleRate(cfg.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    log::info!(
        "opening output stream: format={sample_format:?}, config={stream_config:?}"
    );

    let err_fn = |e| log::error!("audio output stream error: {e}");

    match sample_format {
        SampleFormat::I16 => build_typed_output::<i16>(
            device,
            &stream_config,
            ring,
            err_fn,
            |s| s,
        ),
        SampleFormat::U16 => build_typed_output::<u16>(
            device,
            &stream_config,
            ring,
            err_fn,
            |s| (i32::from(s) + 32_768) as u16,
        ),
        SampleFormat::F32 => build_typed_output::<f32>(
            device,
            &stream_config,
            ring,
            err_fn,
            i16_to_f32,
        ),
        other => bail!("unsupported output sample format: {other:?}"),
    }
}

fn build_typed_output<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    ring: Arc<Mutex<PcmRing>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
    map: impl Fn(i16) -> T + Send + 'static,
) -> Result<cpal::Stream>
where
    T: Sample + SizedSample + FromSample<f32> + Send + 'static,
{
    let mut scratch = Vec::<i16>::new();
    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                scratch.resize(data.len(), 0);
                {
                    let mut ring = ring.lock().unwrap_or_else(|e| e.into_inner());
                    ring.pop_into(&mut scratch);
                }
                for (dst, &src) in data.iter_mut().zip(scratch.iter()) {
                    *dst = map(src);
                }
            },
            err_fn,
            None,
        )
        .context("build_output_stream")?;
    Ok(stream)
}
