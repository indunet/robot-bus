//! Node parameters for the image encoder.

use anyhow::{bail, Context, Result};

use crate::{Node, ParameterValue};

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub input_topic: String,
    pub output_topic: String,
    pub codec: CodecKind,
    pub bitrate: i64,
    pub gop_size: i64,
    pub fps: i64,
    /// Empty → auto-select by [`super::codec::resolve_encoder_name`].
    pub encoder: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecKind {
    H264,
    H265,
}

impl CodecKind {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "h264" | "avc" => Ok(Self::H264),
            "h265" | "hevc" => Ok(Self::H265),
            other => bail!("unsupported codec {other:?}; expected h264 or h265"),
        }
    }

    pub fn as_format(self) -> &'static str {
        match self {
            Self::H264 => "h264",
            Self::H265 => "h265",
        }
    }
}

impl EncoderConfig {
    /// Declare defaults on `node`, optionally overlay a YAML file, then read back.
    pub fn load(node: &mut Node, params_path: Option<&str>) -> Result<Self> {
        declare_defaults(node)?;
        if let Some(path) = params_path {
            node.load_parameters_from_yaml_file(path)
                .with_context(|| format!("load parameters from {path}"))?;
        }
        Self::from_node(node)
    }

    pub fn from_node(node: &Node) -> Result<Self> {
        let input_topic = require_string(node, "input_topic")?;
        let output_topic = require_string(node, "output_topic")?;
        let codec = CodecKind::parse(&require_string(node, "codec")?)?;
        let bitrate = require_i64(node, "bitrate")?;
        let gop_size = require_i64(node, "gop_size")?;
        let fps = require_i64(node, "fps")?;
        let encoder = require_string(node, "encoder")?;
        let width = require_i64(node, "width")?;
        let height = require_i64(node, "height")?;

        if input_topic.is_empty() || output_topic.is_empty() {
            bail!("input_topic and output_topic must be non-empty");
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

        Ok(Self {
            input_topic,
            output_topic,
            codec,
            bitrate,
            gop_size,
            fps,
            encoder,
            width,
            height,
        })
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter(
        "input_topic",
        ParameterValue::String("/camera/image_raw".into()),
    )?;
    node.declare_parameter(
        "output_topic",
        ParameterValue::String("/camera/video".into()),
    )?;
    node.declare_parameter("codec", ParameterValue::String("h264".into()))?;
    node.declare_parameter("bitrate", ParameterValue::Integer(2_000_000))?;
    node.declare_parameter("gop_size", ParameterValue::Integer(30))?;
    node.declare_parameter("fps", ParameterValue::Integer(30))?;
    node.declare_parameter("encoder", ParameterValue::String(String::new()))?;
    node.declare_parameter("width", ParameterValue::Integer(0))?;
    node.declare_parameter("height", ParameterValue::Integer(0))?;
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
    fn codec_parse() {
        assert_eq!(CodecKind::parse("h264").unwrap(), CodecKind::H264);
        assert_eq!(CodecKind::parse("HEVC").unwrap(), CodecKind::H265);
        assert!(CodecKind::parse("vp9").is_err());
    }
}
