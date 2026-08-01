//! Node parameters for the image decoder.

use anyhow::{bail, Context, Result};

use crate::{Node, ParameterValue};

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    pub input_topic: String,
    pub output_topic: String,
    /// Fallback when `CompressedVideo.format` is empty.
    pub codec: CodecKind,
    /// Empty → auto-select by [`super::codec::resolve_decoder_name`].
    pub decoder: String,
    pub output_encoding: OutputEncoding,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEncoding {
    Rgb8,
    Bgr8,
}

impl OutputEncoding {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "rgb8" => Ok(Self::Rgb8),
            "bgr8" => Ok(Self::Bgr8),
            other => bail!("unsupported output_encoding {other:?}; expected rgb8 or bgr8"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rgb8 => "rgb8",
            Self::Bgr8 => "bgr8",
        }
    }
}

impl DecoderConfig {
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
        let decoder = require_string(node, "decoder")?;
        let output_encoding = OutputEncoding::parse(&require_string(node, "output_encoding")?)?;

        if input_topic.is_empty() || output_topic.is_empty() {
            bail!("input_topic and output_topic must be non-empty");
        }

        Ok(Self {
            input_topic,
            output_topic,
            codec,
            decoder,
            output_encoding,
        })
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter(
        "input_topic",
        ParameterValue::String("/camera/video".into()),
    )?;
    node.declare_parameter(
        "output_topic",
        ParameterValue::String("/camera/image_decoded".into()),
    )?;
    node.declare_parameter("codec", ParameterValue::String("h264".into()))?;
    node.declare_parameter("decoder", ParameterValue::String(String::new()))?;
    node.declare_parameter(
        "output_encoding",
        ParameterValue::String("rgb8".into()),
    )?;
    Ok(())
}

fn require_string(node: &Node, name: &str) -> Result<String> {
    match node.get_parameter(name)? {
        ParameterValue::String(s) => Ok(s),
        other => bail!("parameter {name} must be string, got {}", other.type_name()),
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

    #[test]
    fn output_encoding_parse() {
        assert_eq!(OutputEncoding::parse("rgb8").unwrap(), OutputEncoding::Rgb8);
        assert_eq!(OutputEncoding::parse("BGR8").unwrap(), OutputEncoding::Bgr8);
        assert!(OutputEncoding::parse("mono8").is_err());
    }
}
