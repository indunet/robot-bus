//! FFmpeg video encoder → Annex-B access units.

use crate::codec::resolve_encoder_name;
use crate::config::{CodecKind, EncoderConfig};
use crate::convert::image_to_yuv420;
use anyhow::{bail, Context, Result};
use ffmpeg_next::codec::{self, encoder, Flags};
use ffmpeg_next::format::Pixel;
use ffmpeg_next::packet::Packet;
use ffmpeg_next::{Dictionary, Rational};
use robot_bus::sensor_msgs::msg::v1::Image;

pub struct FrameEncoder {
    codec_kind: CodecKind,
    encoder_name: String,
    bitrate: i64,
    gop_size: i64,
    fps: i64,
    force_w: Option<u32>,
    force_h: Option<u32>,
    encoder: Option<encoder::Video>,
    width: u32,
    height: u32,
    pts: i64,
}

impl FrameEncoder {
    pub fn new(cfg: &EncoderConfig) -> Result<Self> {
        ffmpeg_next::init().context("ffmpeg init")?;
        let encoder_name = resolve_encoder_name(cfg.codec, &cfg.encoder)?;
        log::info!(
            "image encoder using FFmpeg encoder {:?} for {}",
            encoder_name,
            cfg.codec.as_format()
        );
        let (force_w, force_h) = if cfg.width > 0 && cfg.height > 0 {
            (Some(cfg.width as u32), Some(cfg.height as u32))
        } else {
            (None, None)
        };
        Ok(Self {
            codec_kind: cfg.codec,
            encoder_name,
            bitrate: cfg.bitrate,
            gop_size: cfg.gop_size,
            fps: cfg.fps,
            force_w,
            force_h,
            encoder: None,
            width: 0,
            height: 0,
            pts: 0,
        })
    }

    pub fn format_str(&self) -> &'static str {
        self.codec_kind.as_format()
    }

    /// Encode one Image; returns Annex-B bytes for one access unit when available.
    pub fn encode_image(&mut self, image: &Image) -> Result<Option<Vec<u8>>> {
        let out_w = self.force_w.unwrap_or(image.width);
        let out_h = self.force_h.unwrap_or(image.height);
        if out_w == 0 || out_h == 0 {
            bail!("encoded frame size is zero");
        }
        // Encoders typically require even dimensions for YUV420.
        let out_w = out_w & !1;
        let out_h = out_h & !1;
        if out_w == 0 || out_h == 0 {
            bail!("encoded frame size rounded to zero");
        }

        if self.encoder.is_none() || self.width != out_w || self.height != out_h {
            self.reopen(out_w, out_h)?;
        }

        let mut yuv = image_to_yuv420(image, out_w, out_h)?;
        yuv.set_pts(Some(self.pts));
        self.pts += 1;

        let enc = self.encoder.as_mut().expect("encoder opened");
        enc.send_frame(&yuv)
            .context("send_frame to video encoder")?;
        self.drain_packets()
    }

    fn reopen(&mut self, width: u32, height: u32) -> Result<()> {
        if let Some(mut old) = self.encoder.take() {
            let _ = old.send_eof();
            let mut pkt = Packet::empty();
            while old.receive_packet(&mut pkt).is_ok() {}
        }

        let codec = encoder::find_by_name(&self.encoder_name)
            .with_context(|| format!("find encoder {}", self.encoder_name))?;

        let mut ctx = codec::context::Context::new_with_codec(codec);
        // Annex-B: do not set GLOBAL_HEADER (avoids AVCC/HVCC extradata-only mode).
        ctx.set_flags(Flags::empty());

        let mut video = ctx.encoder().video().context("encoder video context")?;
        video.set_width(width);
        video.set_height(height);
        video.set_format(Pixel::YUV420P);
        video.set_time_base(Rational::new(1, self.fps as i32));
        video.set_frame_rate(Some(Rational::new(self.fps as i32, 1)));
        video.set_bit_rate(self.bitrate as usize);
        video.set_gop(self.gop_size as u32);
        video.set_max_b_frames(0);

        let mut opts = Dictionary::new();
        match self.encoder_name.as_str() {
            "libx264" => {
                opts.set("preset", "veryfast");
                opts.set("tune", "zerolatency");
                opts.set("annexb", "1");
            }
            "libx265" => {
                opts.set("preset", "ultrafast");
                opts.set("tune", "zerolatency");
                opts.set("x265-params", "annexb=1:bframes=0");
            }
            "libopenh264" => {}
            name if name.contains("nvenc") => {
                opts.set("preset", "p4");
                opts.set("tune", "ll");
                opts.set("bf", "0");
            }
            name if name.contains("videotoolbox") => {
                opts.set("realtime", "true");
            }
            _ => {}
        }

        let opened = video
            .open_as_with(codec, opts)
            .with_context(|| format!("open encoder {}", self.encoder_name))?;

        self.encoder = Some(opened);
        self.width = width;
        self.height = height;
        self.pts = 0;
        log::info!("opened encoder {} at {width}x{height}", self.encoder_name);
        Ok(())
    }

    fn drain_packets(&mut self) -> Result<Option<Vec<u8>>> {
        let enc = self.encoder.as_mut().expect("encoder opened");
        let mut out: Option<Vec<u8>> = None;
        loop {
            let mut packet = Packet::empty();
            match enc.receive_packet(&mut packet) {
                Ok(()) => {
                    if let Some(data) = packet.data() {
                        out.get_or_insert_with(Vec::new).extend_from_slice(data);
                    }
                }
                Err(ffmpeg_next::Error::Other {
                    errno: ffmpeg_next::error::EAGAIN,
                })
                | Err(ffmpeg_next::Error::Eof) => break,
                Err(e) => return Err(e).context("receive_packet from video encoder"),
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robot_bus::sensor_msgs::msg::v1::Image;

    fn solid_rgb8(width: u32, height: u32, r: u8, g: u8, b: u8) -> Image {
        let mut data = Vec::with_capacity((width * height * 3) as usize);
        for _ in 0..(width * height) {
            data.push(r);
            data.push(g);
            data.push(b);
        }
        Image {
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
    fn encode_one_rgb_frame_produces_annexb() {
        let cfg = EncoderConfig {
            input_topic: "/in".into(),
            output_topic: "/out".into(),
            codec: CodecKind::H264,
            bitrate: 500_000,
            gop_size: 30,
            fps: 30,
            // Prefer soft encoder for CI/dev machines without NVENC.
            encoder: "libx264".into(),
            width: 0,
            height: 0,
        };

        // Skip cleanly when this FFmpeg build has no libx264 (and no forced fallback).
        if ffmpeg_next::init().is_err() {
            return;
        }
        if ffmpeg_next::encoder::find_by_name("libx264").is_none() {
            // Try whatever auto-resolve picks.
            let cfg = EncoderConfig {
                encoder: String::new(),
                ..cfg
            };
            let Ok(mut enc) = FrameEncoder::new(&cfg) else {
                return;
            };
            let img = solid_rgb8(64, 48, 10, 20, 30);
            let out = enc.encode_image(&img).expect("encode");
            // First frame may be delayed; feed a few.
            let mut bytes = out.unwrap_or_default();
            for _ in 0..5 {
                if let Some(more) = enc.encode_image(&img).expect("encode more") {
                    bytes.extend(more);
                }
                if !bytes.is_empty() {
                    break;
                }
            }
            assert!(!bytes.is_empty(), "expected some encoded bytes");
            return;
        }

        let mut enc = FrameEncoder::new(&cfg).expect("open libx264 encoder");
        let img = solid_rgb8(64, 48, 10, 20, 30);
        let mut bytes = Vec::new();
        for _ in 0..8 {
            if let Some(chunk) = enc.encode_image(&img).expect("encode") {
                bytes.extend(chunk);
            }
            if bytes.len() > 4 {
                break;
            }
        }
        assert!(
            bytes.windows(4).any(|w| w == [0, 0, 0, 1] || w == [0, 0, 1]),
            "expected Annex-B start code in bitstream, got {} bytes",
            bytes.len()
        );
    }
}
