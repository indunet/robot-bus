//! Encoder name resolution (hardware / non-GPL preferred).

use crate::config::CodecKind;
use anyhow::{bail, Result};

/// Candidates in preference order for `codec`.
pub fn encoder_candidates(codec: CodecKind) -> &'static [&'static str] {
    match codec {
        CodecKind::H264 => &[
            "h264_nvenc",
            "h264_videotoolbox",
            "libopenh264",
            "libx264",
        ],
        CodecKind::H265 => &["hevc_nvenc", "hevc_videotoolbox", "libx265"],
    }
}

/// Resolve the FFmpeg encoder name: forced `encoder` override, else first available candidate.
pub fn resolve_encoder_name(codec: CodecKind, forced: &str) -> Result<String> {
    let forced = forced.trim();
    if !forced.is_empty() {
        if ffmpeg_next::encoder::find_by_name(forced).is_none() {
            bail!("requested encoder {forced:?} is not available in this FFmpeg build");
        }
        return Ok(forced.to_string());
    }

    for name in encoder_candidates(codec) {
        if ffmpeg_next::encoder::find_by_name(name).is_some() {
            return Ok((*name).to_string());
        }
    }

    bail!(
        "no encoder available for {}; tried {:?}",
        codec.as_format(),
        encoder_candidates(codec)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h264_candidates_prefer_hardware() {
        let c = encoder_candidates(CodecKind::H264);
        assert_eq!(c[0], "h264_nvenc");
        assert_eq!(c.last().copied(), Some("libx264"));
    }

    #[test]
    fn h265_candidates_prefer_hardware() {
        let c = encoder_candidates(CodecKind::H265);
        assert_eq!(c[0], "hevc_nvenc");
        assert!(c.contains(&"libx265"));
    }
}
