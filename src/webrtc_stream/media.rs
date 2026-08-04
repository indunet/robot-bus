//! Image → H.264 (FrameEncoder) and RawAudio → Opus.

use anyhow::{bail, Context, Result};
use std::time::Duration;

use super::config::WebrtcConfig;
use crate::foxglove_msgs::msg::v1::RawAudio;
use crate::image_encoder::encoder::FrameEncoder;
use crate::sensor_msgs::msg::v1::Image;

/// Encode `sensor_msgs/Image` frames to Annex-B H.264 access units.
pub struct VideoPipeline {
    encoder: FrameEncoder,
    frame_duration: Duration,
}

impl VideoPipeline {
    pub fn new(cfg: &WebrtcConfig) -> Result<Self> {
        let encoder = FrameEncoder::new(&cfg.encoder_config())?;
        let frame_duration = Duration::from_secs_f64(1.0 / cfg.fps as f64);
        Ok(Self {
            encoder,
            frame_duration,
        })
    }

    pub fn encode(&mut self, image: &Image) -> Result<Option<(Vec<u8>, Duration)>> {
        match self.encoder.encode_image(image)? {
            Some(data) if !data.is_empty() => Ok(Some((data, self.frame_duration))),
            _ => Ok(None),
        }
    }
}

/// Encode `foxglove_msgs/RawAudio` (pcm-s16) to Opus packets at 48 kHz.
pub struct AudioPipeline {
    encoder: opus::Encoder,
    expected_rate: u32,
    expected_channels: u16,
    /// Interleaved PCM at 48 kHz awaiting a full Opus frame.
    pending_48k: Vec<i16>,
    /// Samples per Opus frame at 48 kHz (20 ms).
    frame_samples: usize,
    channels: usize,
}

impl AudioPipeline {
    pub fn new(cfg: &WebrtcConfig) -> Result<Self> {
        let channels = match cfg.channels {
            1 => opus::Channels::Mono,
            2 => opus::Channels::Stereo,
            _ => bail!("channels must be 1 or 2"),
        };
        let mut encoder =
            opus::Encoder::new(48_000, channels, opus::Application::Voip).context("opus encoder")?;
        encoder
            .set_bitrate(opus::Bitrate::Bits(cfg.opus_bitrate as i32))
            .context("set opus bitrate")?;
        let ch = cfg.channels as usize;
        Ok(Self {
            encoder,
            expected_rate: cfg.sample_rate,
            expected_channels: cfg.channels,
            pending_48k: Vec::with_capacity(48_000 / 50 * ch),
            frame_samples: 48_000 / 50 * ch, // 20 ms
            channels: ch,
        })
    }

    /// Returns zero or more Opus packets (each ~20 ms).
    pub fn push(&mut self, msg: &RawAudio) -> Result<Vec<(Vec<u8>, Duration)>> {
        if msg.format != "pcm-s16" {
            bail!("unsupported format {:?}; expected pcm-s16", msg.format);
        }
        if msg.sample_rate != self.expected_rate {
            bail!(
                "sample_rate mismatch: msg={} expected={}",
                msg.sample_rate,
                self.expected_rate
            );
        }
        if msg.number_of_channels as u16 != self.expected_channels {
            bail!(
                "channels mismatch: msg={} expected={}",
                msg.number_of_channels,
                self.expected_channels
            );
        }
        if msg.data.len() % 2 != 0 {
            bail!("pcm-s16 payload length must be even");
        }

        let pcm: Vec<i16> = msg
            .data
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect();
        let up = upsample_to_48k(&pcm, self.expected_rate, self.channels)?;
        self.pending_48k.extend_from_slice(&up);

        let mut out = Vec::new();
        let frame_dur = Duration::from_millis(20);
        while self.pending_48k.len() >= self.frame_samples {
            let frame: Vec<i16> = self.pending_48k.drain(..self.frame_samples).collect();
            let mut buf = vec![0u8; 4000];
            let n = self
                .encoder
                .encode(&frame, &mut buf)
                .context("opus encode")?;
            buf.truncate(n);
            if !buf.is_empty() {
                out.push((buf, frame_dur));
            }
        }
        Ok(out)
    }
}

fn upsample_to_48k(pcm: &[i16], rate: u32, channels: usize) -> Result<Vec<i16>> {
    if rate == 48_000 {
        return Ok(pcm.to_vec());
    }
    if 48_000 % rate != 0 {
        bail!("sample_rate {rate} does not divide 48000");
    }
    let factor = (48_000 / rate) as usize;
    if pcm.len() % channels != 0 {
        bail!("pcm length not divisible by channel count");
    }
    let frames = pcm.len() / channels;
    let mut out = Vec::with_capacity(pcm.len() * factor);
    for i in 0..frames {
        let base = i * channels;
        for _ in 0..factor {
            out.extend_from_slice(&pcm[base..base + channels]);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsample_16k_mono() {
        let pcm = vec![1i16, 2, 3];
        let out = upsample_to_48k(&pcm, 16_000, 1).unwrap();
        assert_eq!(out, vec![1, 1, 1, 2, 2, 2, 3, 3, 3]);
    }
}
