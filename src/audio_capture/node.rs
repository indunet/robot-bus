//! Wire robot-bus Node: microphone → RawAudio out.

use super::config::CaptureConfig;
use super::device::resolve_input_device;
use super::pcm::{append_i16_le, f32_to_i16, frames_per_chunk, u16_to_i16};
use anyhow::{bail, Context, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig};
use prost_types::Timestamp;
use crate::foxglove_msgs::msg::v1::RawAudio;
use crate::{Node, NodeOptions, TopicPublisher};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Mutex-guarded publisher shared with the cpal callback thread.
struct SharedPub {
    inner: Mutex<TopicPublisher<RawAudio>>,
}

// Safety: all socket use is serialized by `inner`.
unsafe impl Send for SharedPub {}
unsafe impl Sync for SharedPub {}

struct ChunkState {
    /// Interleaved i16 samples waiting to form a full chunk.
    pending: Vec<i16>,
    frames_needed: usize,
    channels: usize,
    sample_rate: u32,
    scratch_i16: Vec<i16>,
}

/// Build the node, open the input stream, publish chunks, and spin.
pub fn run(node_name: &str, options: NodeOptions, params_path: Option<&str>) -> Result<()> {
    let mut node = Node::with_options(node_name, options);
    let cfg = CaptureConfig::load(&mut node, params_path)?;
    let device = resolve_input_device(&cfg.device)?;
    let device_name = device.name().unwrap_or_else(|_| "<unknown>".into());

    log::info!(
        "audio capture ready: device={device_name:?} -> {} ({} Hz, {} ch, {} ms)",
        cfg.output_topic,
        cfg.sample_rate,
        cfg.channels,
        cfg.chunk_ms
    );

    let publisher = Arc::new(SharedPub {
        inner: Mutex::new(
            node.create_publisher::<RawAudio>(&cfg.output_topic)
                .context("create RawAudio publisher")?,
        ),
    });

    let frames_needed = frames_per_chunk(cfg.sample_rate, cfg.chunk_ms);
    let state = Arc::new(Mutex::new(ChunkState {
        pending: Vec::with_capacity(frames_needed * cfg.channels as usize),
        frames_needed,
        channels: cfg.channels as usize,
        sample_rate: cfg.sample_rate,
        scratch_i16: Vec::new(),
    }));

    let stream = build_input_stream(&device, &cfg, publisher, state)?;
    stream.play().context("start input stream")?;

    // Keep stream alive for the lifetime of spin.
    node.spin().context("node spin")?;
    drop(stream);
    Ok(())
}

fn build_input_stream(
    device: &cpal::Device,
    cfg: &CaptureConfig,
    publisher: Arc<SharedPub>,
    state: Arc<Mutex<ChunkState>>,
) -> Result<cpal::Stream> {
    let supported = device
        .default_input_config()
        .context("default input config")?;

    let sample_format = supported.sample_format();
    let stream_config = StreamConfig {
        channels: cfg.channels,
        sample_rate: cpal::SampleRate(cfg.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    log::info!(
        "opening input stream: format={sample_format:?}, config={stream_config:?}"
    );

    let err_fn = |e| log::error!("audio input stream error: {e}");

    match sample_format {
        SampleFormat::I16 => build_typed_stream::<i16>(
            device,
            &stream_config,
            publisher,
            state,
            err_fn,
            |data, scratch| {
                scratch.clear();
                scratch.extend_from_slice(data);
            },
        ),
        SampleFormat::U16 => build_typed_stream::<u16>(
            device,
            &stream_config,
            publisher,
            state,
            err_fn,
            u16_to_i16,
        ),
        SampleFormat::F32 => build_typed_stream::<f32>(
            device,
            &stream_config,
            publisher,
            state,
            err_fn,
            f32_to_i16,
        ),
        other => bail!("unsupported input sample format: {other:?}"),
    }
}

fn build_typed_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    publisher: Arc<SharedPub>,
    state: Arc<Mutex<ChunkState>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
    convert: impl Fn(&[T], &mut Vec<i16>) + Send + 'static,
) -> Result<cpal::Stream>
where
    T: Sample + SizedSample + FromSample<f32> + Send + 'static,
{
    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                convert(data, &mut st.scratch_i16);
                let converted = std::mem::take(&mut st.scratch_i16);
                st.pending.extend_from_slice(&converted);
                st.scratch_i16 = converted;

                let samples_per_chunk = st.frames_needed.saturating_mul(st.channels);
                while st.pending.len() >= samples_per_chunk {
                    let chunk: Vec<i16> = st.pending.drain(..samples_per_chunk).collect();
                    let sample_rate = st.sample_rate;
                    let channels = st.channels as u32;
                    drop(st);

                    if let Err(e) = publish_chunk(&publisher, &chunk, sample_rate, channels) {
                        log::warn!("publish RawAudio failed: {e:#}");
                    }

                    st = state.lock().unwrap_or_else(|e| e.into_inner());
                }
            },
            err_fn,
            None,
        )
        .context("build_input_stream")?;
    Ok(stream)
}

fn publish_chunk(
    publisher: &SharedPub,
    samples: &[i16],
    sample_rate: u32,
    channels: u32,
) -> Result<()> {
    let mut data = Vec::with_capacity(samples.len() * 2);
    append_i16_le(&mut data, samples);

    let msg = RawAudio {
        timestamp: Some(now_timestamp()),
        data,
        format: "pcm-s16".into(),
        sample_rate,
        number_of_channels: channels,
    };

    let pub_ = publisher
        .inner
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    pub_.publish(&msg).context("publish RawAudio")?;
    Ok(())
}

fn now_timestamp() -> Timestamp {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    Timestamp {
        seconds: dur.as_secs() as i64,
        nanos: dur.subsec_nanos() as i32,
    }
}
