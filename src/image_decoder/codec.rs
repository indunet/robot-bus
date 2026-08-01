//! Decoder name resolution (hardware preferred).

use anyhow::{bail, Result};

use super::config::CodecKind;

/// Candidates in preference order for `codec`.
pub fn decoder_candidates(codec: CodecKind) -> &'static [&'static str] {
    match codec {
        CodecKind::H264 => &["h264_cuvid", "h264_videotoolbox", "h264"],
        CodecKind::H265 => &["hevc_cuvid", "hevc_videotoolbox", "hevc"],
    }
}

/// Resolve the FFmpeg decoder name: forced `decoder` override, else first available candidate.
pub fn resolve_decoder_name(codec: CodecKind, forced: &str) -> Result<String> {
    let forced = forced.trim();
    if !forced.is_empty() {
        if ffmpeg_next::decoder::find_by_name(forced).is_none() {
            bail!("requested decoder {forced:?} is not available in this FFmpeg build");
        }
        return Ok(forced.to_string());
    }

    for name in decoder_candidates(codec) {
        if ffmpeg_next::decoder::find_by_name(name).is_some() {
            return Ok((*name).to_string());
        }
    }

    bail!(
        "no decoder available for {}; tried {:?}",
        codec.as_format(),
        decoder_candidates(codec)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h264_candidates_prefer_hardware() {
        let c = decoder_candidates(CodecKind::H264);
        assert_eq!(c[0], "h264_cuvid");
        assert_eq!(c.last().copied(), Some("h264"));
    }

    #[test]
    fn h265_candidates_prefer_hardware() {
        let c = decoder_candidates(CodecKind::H265);
        assert_eq!(c[0], "hevc_cuvid");
        assert!(c.contains(&"hevc"));
    }
}
