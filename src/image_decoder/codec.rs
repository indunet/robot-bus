//! Decoder name resolution (hardware preferred, open-probed).

use anyhow::{bail, Context, Result};
use ffmpeg_next::codec;
use ffmpeg_next::decoder;

use super::config::CodecKind;

/// Candidates in preference order for `codec`.
pub fn decoder_candidates(codec: CodecKind) -> &'static [&'static str] {
    match codec {
        CodecKind::H264 => &["h264_cuvid", "h264_videotoolbox", "h264"],
        CodecKind::H265 => &["hevc_cuvid", "hevc_videotoolbox", "hevc"],
    }
}

/// Open a named FFmpeg video decoder.
pub fn open_video_decoder(name: &str) -> Result<decoder::Video> {
    let codec_obj = decoder::find_by_name(name)
        .with_context(|| format!("find decoder {name}"))?;
    let ctx = codec::context::Context::new_with_codec(codec_obj);
    ctx.decoder()
        .open_as(codec_obj)
        .with_context(|| format!("open decoder {name}"))?
        .video()
        .with_context(|| format!("decoder {name} is not a video decoder"))
}

/// Resolve and open: forced override, else first candidate that actually opens.
///
/// Ubuntu FFmpeg often lists `*_cuvid` even without an NVIDIA device; `find_by_name`
/// succeeds but `open` fails with EPERM. Probe by opening so we fall back to software.
pub fn open_decoder(codec: CodecKind, forced: &str) -> Result<(String, decoder::Video)> {
    let forced = forced.trim();
    if !forced.is_empty() {
        let opened = open_video_decoder(forced)?;
        return Ok((forced.to_string(), opened));
    }

    let mut errors = Vec::new();
    for name in decoder_candidates(codec) {
        match open_video_decoder(name) {
            Ok(opened) => return Ok(((*name).to_string(), opened)),
            Err(e) => {
                log::debug!("skipping decoder {name}: {e:#}");
                errors.push(format!("{name}: {e:#}"));
            }
        }
    }

    bail!(
        "no decoder available for {}; tried: {}",
        codec.as_format(),
        errors.join("; ")
    )
}

/// Resolve the FFmpeg decoder name by probing open (then dropping the instance).
pub fn resolve_decoder_name(codec: CodecKind, forced: &str) -> Result<String> {
    let (name, _opened) = open_decoder(codec, forced)?;
    Ok(name)
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

    #[test]
    fn resolve_auto_picks_an_openable_decoder() {
        if ffmpeg_next::init().is_err() {
            return;
        }
        let Ok(name) = resolve_decoder_name(CodecKind::H264, "") else {
            return;
        };
        // Must be able to open again (not a phantom like cuvid-without-GPU).
        open_video_decoder(&name).expect("resolved decoder must open");
    }
}
