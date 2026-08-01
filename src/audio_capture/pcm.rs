//! PCM helpers: sample conversion to interleaved little-endian `pcm-s16`.

/// Number of frames in one chunk of `chunk_ms` at `sample_rate`.
pub fn frames_per_chunk(sample_rate: u32, chunk_ms: u32) -> usize {
    let n = u64::from(sample_rate) * u64::from(chunk_ms) / 1000;
    n.max(1) as usize
}

/// Append interleaved i16 samples as little-endian bytes.
pub fn append_i16_le(dst: &mut Vec<u8>, samples: &[i16]) {
    dst.reserve(samples.len() * 2);
    for &s in samples {
        dst.extend_from_slice(&s.to_le_bytes());
    }
}

/// Convert a buffer of interleaved f32 samples (−1.0..1.0) to i16.
pub fn f32_to_i16(samples: &[f32], out: &mut Vec<i16>) {
    out.clear();
    out.reserve(samples.len());
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let v = (clamped * f32::from(i16::MAX)).round() as i16;
        out.push(v);
    }
}

/// Convert interleaved u16 (0..65535 centered at 32768) to i16.
pub fn u16_to_i16(samples: &[u16], out: &mut Vec<i16>) {
    out.clear();
    out.reserve(samples.len());
    for &s in samples {
        out.push((i32::from(s) - 32_768) as i16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_frames() {
        assert_eq!(frames_per_chunk(16_000, 20), 320);
        assert_eq!(frames_per_chunk(48_000, 10), 480);
        assert_eq!(frames_per_chunk(8_000, 0), 1);
    }

    #[test]
    fn i16_le_bytes() {
        let mut buf = Vec::new();
        append_i16_le(&mut buf, &[0x0102_i16, -2]);
        assert_eq!(buf, vec![0x02, 0x01, 0xfe, 0xff]);
    }

    #[test]
    fn f32_conversion() {
        let mut out = Vec::new();
        f32_to_i16(&[0.0, 1.0, -1.0], &mut out);
        assert_eq!(out[0], 0);
        assert_eq!(out[1], i16::MAX);
        assert_eq!(out[2], -i16::MAX);
    }

    #[test]
    fn u16_conversion() {
        let mut out = Vec::new();
        u16_to_i16(&[32_768, 32_769, 32_767], &mut out);
        assert_eq!(out, vec![0, 1, -1]);
    }
}
