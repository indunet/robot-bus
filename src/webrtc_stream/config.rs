//! Node parameters for the WebRTC / WHEP streamer.

use anyhow::{bail, Context, Result};
use std::net::SocketAddr;

use crate::{Node, ParameterValue};

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct WebrtcConfig {
    pub image_topic: String,
    pub audio_topic: String,
    pub data_topics: Vec<String>,
    pub listen: SocketAddr,
    pub bitrate: i64,
    pub gop_size: i64,
    pub fps: i64,
    pub encoder: String,
    pub width: i64,
    pub height: i64,
    pub sample_rate: u32,
    pub channels: u16,
    pub opus_bitrate: i64,
}

impl WebrtcConfig {
    pub fn load(node: &mut Node, params_path: Option<&str>) -> Result<Self> {
        declare_defaults(node)?;
        if let Some(path) = params_path {
            node.load_parameters_from_yaml_file(path)
                .with_context(|| format!("load parameters from {path}"))?;
        }
        Self::from_node(node)
    }

    pub fn from_node(node: &Node) -> Result<Self> {
        let image_topic = require_string(node, "image_topic")?;
        let audio_topic = require_string(node, "audio_topic")?;
        let data_topics = parse_topic_list(&require_string(node, "data_topics")?)?;
        let listen: SocketAddr = require_string(node, "listen")?
            .parse()
            .context("parse listen address")?;
        let bitrate = require_i64(node, "bitrate")?;
        let gop_size = require_i64(node, "gop_size")?;
        let fps = require_i64(node, "fps")?;
        let encoder = require_string(node, "encoder")?;
        let width = require_i64(node, "width")?;
        let height = require_i64(node, "height")?;
        let sample_rate = require_i64(node, "sample_rate")? as u32;
        let channels = require_i64(node, "channels")? as u16;
        let opus_bitrate = require_i64(node, "opus_bitrate")?;

        if image_topic.is_empty() && audio_topic.is_empty() && data_topics.is_empty() {
            bail!("at least one of image_topic, audio_topic, or data_topics must be set");
        }
        if bitrate <= 0 {
            bail!("bitrate must be > 0");
        }
        if gop_size <= 0 {
            bail!("gop_size must be > 0");
        }
        if fps <= 0 {
            bail!("fps must be > 0");
        }
        if width < 0 || height < 0 {
            bail!("width/height must be >= 0 (0 = follow input)");
        }
        if (width == 0) != (height == 0) {
            bail!("width and height must both be 0 or both non-zero");
        }
        if !matches!(sample_rate, 8_000 | 16_000 | 24_000 | 48_000) {
            bail!("sample_rate must be 8000, 16000, 24000, or 48000");
        }
        if channels == 0 || channels > 2 {
            bail!("channels must be 1 or 2");
        }
        if opus_bitrate <= 0 {
            bail!("opus_bitrate must be > 0");
        }

        Ok(Self {
            image_topic,
            audio_topic,
            data_topics,
            listen,
            bitrate,
            gop_size,
            fps,
            encoder,
            width,
            height,
            sample_rate,
            channels,
            opus_bitrate,
        })
    }

    /// Build an [`crate::image_encoder::EncoderConfig`] for [`FrameEncoder`](crate::image_encoder::encoder::FrameEncoder).
    pub fn encoder_config(&self) -> crate::image_encoder::config::EncoderConfig {
        use crate::image_encoder::config::{CodecKind, EncoderConfig};
        EncoderConfig {
            input_topic: self.image_topic.clone(),
            output_topic: "/webrtc/unused".into(),
            codec: CodecKind::H264,
            bitrate: self.bitrate,
            gop_size: self.gop_size,
            fps: self.fps,
            encoder: self.encoder.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

fn parse_topic_list(raw: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let t = part.trim();
        if t.is_empty() {
            continue;
        }
        if !t.starts_with('/') {
            bail!("data topic must start with '/': {t:?}");
        }
        out.push(t.to_string());
    }
    Ok(out)
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter(
        "image_topic",
        ParameterValue::String("/camera/image_raw".into()),
    )?;
    node.declare_parameter("audio_topic", ParameterValue::String("/audio/mic".into()))?;
    node.declare_parameter("data_topics", ParameterValue::String(String::new()))?;
    node.declare_parameter(
        "listen",
        ParameterValue::String("0.0.0.0:8090".into()),
    )?;
    node.declare_parameter("bitrate", ParameterValue::Integer(2_000_000))?;
    node.declare_parameter("gop_size", ParameterValue::Integer(30))?;
    node.declare_parameter("fps", ParameterValue::Integer(30))?;
    node.declare_parameter("encoder", ParameterValue::String(String::new()))?;
    node.declare_parameter("width", ParameterValue::Integer(0))?;
    node.declare_parameter("height", ParameterValue::Integer(0))?;
    node.declare_parameter("sample_rate", ParameterValue::Integer(16_000))?;
    node.declare_parameter("channels", ParameterValue::Integer(1))?;
    node.declare_parameter("opus_bitrate", ParameterValue::Integer(32_000))?;
    Ok(())
}

fn require_string(node: &Node, name: &str) -> Result<String> {
    match node.get_parameter(name)? {
        ParameterValue::String(s) => Ok(s),
        other => bail!("parameter {name} must be string, got {}", other.type_name()),
    }
}

fn require_i64(node: &Node, name: &str) -> Result<i64> {
    match node.get_parameter(name)? {
        ParameterValue::Integer(i) => Ok(i),
        ParameterValue::Double(d) if d.fract() == 0.0 => Ok(d as i64),
        other => bail!(
            "parameter {name} must be integer, got {}",
            other.type_name()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_topics() {
        assert!(parse_topic_list("").unwrap().is_empty());
        assert_eq!(
            parse_topic_list("/a, /b").unwrap(),
            vec!["/a".to_string(), "/b".to_string()]
        );
        assert!(parse_topic_list("no_slash").is_err());
    }
}
