//! PCM helpers for validating and decoding `pcm-s16` payloads.

use anyhow::{bail, Result};

/// Decode interleaved little-endian i16 samples from a RawAudio payload.
pub fn decode_pcm_s16(data: &[u8]) -> Result<Vec<i16>> {
    if !data.len().is_multiple_of(2) {
        bail!("pcm-s16 payload length must be even, got {}", data.len());
    }
    let mut out = Vec::with_capacity(data.len() / 2);
    for chunk in data.chunks_exact(2) {
        out.push(i16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Ok(out)
}

/// Max interleaved sample count for a buffer of `max_buffer_ms` at the given rate/channels.
pub fn max_samples(sample_rate: u32, channels: u16, max_buffer_ms: u32) -> usize {
    let frames = u64::from(sample_rate) * u64::from(max_buffer_ms) / 1000;
    (frames.max(1) as usize).saturating_mul(channels as usize)
}

/// Convert i16 samples to f32 (−1.0..1.0) for device output.
pub fn i16_to_f32(sample: i16) -> f32 {
    f32::from(sample) / f32::from(i16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_roundtrip() {
        let bytes = [0x02, 0x01, 0xfe, 0xff];
        let samples = decode_pcm_s16(&bytes).unwrap();
        assert_eq!(samples, vec![0x0102_i16, -2]);
    }

    #[test]
    fn decode_odd_fails() {
        assert!(decode_pcm_s16(&[1]).is_err());
    }

    #[test]
    fn buffer_cap() {
        assert_eq!(max_samples(16_000, 1, 500), 8_000);
        assert_eq!(max_samples(16_000, 2, 100), 3_200);
    }

    #[test]
    fn f32_scale() {
        assert!((i16_to_f32(0) - 0.0).abs() < 1e-6);
        assert!((i16_to_f32(i16::MAX) - 1.0).abs() < 1e-6);
    }
}
