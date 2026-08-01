//! FFmpeg video decoder: Annex-B access units → Image.

use anyhow::{Context, Result};
use ffmpeg_next::decoder;
use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::packet::Packet;

use super::codec::{open_decoder, resolve_decoder_name};
use super::config::{CodecKind, DecoderConfig, OutputEncoding};
use super::convert::frame_to_image;
use crate::builtin_interfaces::msg::v1::Time;
use crate::sensor_msgs::msg::v1::Image;

pub struct FrameDecoder {
    fallback_codec: CodecKind,
    forced_decoder: String,
    output_encoding: OutputEncoding,
    decoder_name: Option<String>,
    active_codec: Option<CodecKind>,
    decoder: Option<decoder::Video>,
}

impl FrameDecoder {
    pub fn new(cfg: &DecoderConfig) -> Result<Self> {
        ffmpeg_next::init().context("ffmpeg init")?;
        // Resolve once so we fail fast when no decoder is available for the default codec.
        let name = resolve_decoder_name(cfg.codec, &cfg.decoder)?;
        log::info!(
            "image decoder using FFmpeg decoder {:?} for {} (message format may override)",
            name,
            cfg.codec.as_format()
        );
        Ok(Self {
            fallback_codec: cfg.codec,
            forced_decoder: cfg.decoder.clone(),
            output_encoding: cfg.output_encoding,
            decoder_name: None,
            active_codec: None,
            decoder: None,
        })
    }

    /// Decode one Annex-B access unit. Returns `None` until a full frame is available
    /// (e.g. waiting for the first keyframe).
    pub fn decode_access_unit(
        &mut self,
        data: &[u8],
        format: &str,
        frame_id: &str,
        stamp: Option<Time>,
    ) -> Result<Option<Image>> {
        if data.is_empty() {
            return Ok(None);
        }

        let codec = if format.trim().is_empty() {
            self.fallback_codec
        } else {
            CodecKind::parse(format)?
        };

        if self.decoder.is_none() || self.active_codec != Some(codec) {
            self.reopen(codec)?;
        }

        let packet = Packet::copy(data);
        let dec = self.decoder.as_mut().expect("decoder opened");
        match dec.send_packet(&packet) {
            Ok(()) => {}
            Err(ffmpeg_next::Error::Other {
                errno: ffmpeg_next::error::EAGAIN,
            }) => {
                // Decoder is full; drain first then retry once.
                let _ = self.drain_frames(frame_id, stamp.clone())?;
                let dec = self.decoder.as_mut().expect("decoder opened");
                dec.send_packet(&packet)
                    .context("send_packet to video decoder after drain")?;
            }
            Err(e) => return Err(e).context("send_packet to video decoder"),
        }

        self.drain_frames(frame_id, stamp)
    }

    fn reopen(&mut self, codec: CodecKind) -> Result<()> {
        if let Some(mut old) = self.decoder.take() {
            let _ = old.send_eof();
            let mut discarded = VideoFrame::empty();
            while old.receive_frame(&mut discarded).is_ok() {}
        }

        let (decoder_name, opened) = open_decoder(codec, &self.forced_decoder)?;

        log::info!("opened decoder {decoder_name} for {}", codec.as_format());
        self.decoder_name = Some(decoder_name);
        self.active_codec = Some(codec);
        self.decoder = Some(opened);
        Ok(())
    }

    fn drain_frames(&mut self, frame_id: &str, stamp: Option<Time>) -> Result<Option<Image>> {
        let dec = self.decoder.as_mut().expect("decoder opened");
        let mut last: Option<VideoFrame> = None;
        loop {
            let mut frame = VideoFrame::empty();
            match dec.receive_frame(&mut frame) {
                Ok(()) => last = Some(frame),
                Err(ffmpeg_next::Error::Other {
                    errno: ffmpeg_next::error::EAGAIN,
                })
                | Err(ffmpeg_next::Error::Eof) => break,
                Err(e) => return Err(e).context("receive_frame from video decoder"),
            }
        }

        match last {
            Some(frame) => Ok(Some(frame_to_image(
                &frame,
                self.output_encoding,
                frame_id,
                stamp,
            )?)),
            None => Ok(None),
        }
    }
}

#[cfg(all(test, feature = "image-encoder"))]
mod tests {
    use super::*;
    use crate::image_encoder::config::{CodecKind as EncCodec, EncoderConfig};
    use crate::image_encoder::encoder::FrameEncoder;
    use crate::sensor_msgs::msg::v1::Image as EncImage;

    fn solid_rgb8(width: u32, height: u32, r: u8, g: u8, b: u8) -> EncImage {
        let mut data = Vec::with_capacity((width * height * 3) as usize);
        for _ in 0..(width * height) {
            data.push(r);
            data.push(g);
            data.push(b);
        }
        EncImage {
            height,
            width,
            encoding: "rgb8".into(),
            is_bigendian: false,
            step: width * 3,
            data,
            ..Default::default()
        }
    }

    #[test]
    fn encode_then_decode_roundtrip() {
        if ffmpeg_next::init().is_err() {
            return;
        }

        let enc_cfg = EncoderConfig {
            input_topic: "/in".into(),
            output_topic: "/out".into(),
            codec: EncCodec::H264,
            bitrate: 500_000,
            gop_size: 30,
            fps: 30,
            encoder: "libx264".into(),
            width: 0,
            height: 0,
        };

        let mut enc = if ffmpeg_next::encoder::find_by_name("libx264").is_some() {
            FrameEncoder::new(&enc_cfg).expect("open libx264")
        } else {
            let cfg = EncoderConfig {
                encoder: String::new(),
                ..enc_cfg
            };
            let Ok(enc) = FrameEncoder::new(&cfg) else {
                return;
            };
            enc
        };

        // Force software decode so CI (FFmpeg lists h264_cuvid but cannot open it) is stable.
        let dec_cfg = DecoderConfig {
            input_topic: "/out".into(),
            output_topic: "/decoded".into(),
            codec: CodecKind::H264,
            decoder: "h264".into(),
            output_encoding: OutputEncoding::Rgb8,
        };
        let Ok(mut dec) = FrameDecoder::new(&dec_cfg) else {
            return;
        };

        let img = solid_rgb8(64, 48, 10, 20, 30);
        let mut decoded = None;
        for _ in 0..16 {
            if let Some(chunk) = enc.encode_image(&img).expect("encode") {
                if let Some(out) = dec
                    .decode_access_unit(&chunk, "h264", "cam", None)
                    .expect("decode")
                {
                    decoded = Some(out);
                    break;
                }
            }
        }

        let out = decoded.expect("expected a decoded frame after feeding keyframes");
        assert_eq!(out.width, 64);
        assert_eq!(out.height, 48);
        assert_eq!(out.encoding, "rgb8");
        assert_eq!(out.data.len(), 64 * 48 * 3);
    }
}
