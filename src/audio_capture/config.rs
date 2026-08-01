//! Node parameters for the audio capture tool node.

use anyhow::{bail, Context, Result};
use crate::{Node, ParameterValue};

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub output_topic: String,
    /// Empty → system default input device.
    pub device: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_ms: u32,
}

impl CaptureConfig {
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
        let output_topic = require_string(node, "output_topic")?;
        let device = require_string(node, "device")?;
        let sample_rate = require_u32(node, "sample_rate")?;
        let channels = require_u16(node, "channels")?;
        let chunk_ms = require_u32(node, "chunk_ms")?;

        if output_topic.is_empty() {
            bail!("output_topic must be non-empty");
        }
        if sample_rate == 0 {
            bail!("sample_rate must be > 0");
        }
        if channels == 0 {
            bail!("channels must be > 0");
        }
        if chunk_ms == 0 {
            bail!("chunk_ms must be > 0");
        }

        Ok(Self {
            output_topic,
            device,
            sample_rate,
            channels,
            chunk_ms,
        })
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter(
        "output_topic",
        ParameterValue::String("/audio/mic".into()),
    )?;
    node.declare_parameter("device", ParameterValue::String(String::new()))?;
    node.declare_parameter("sample_rate", ParameterValue::Integer(16_000))?;
    node.declare_parameter("channels", ParameterValue::Integer(1))?;
    node.declare_parameter("chunk_ms", ParameterValue::Integer(20))?;
    Ok(())
}

fn require_string(node: &Node, name: &str) -> Result<String> {
    match node.get_parameter(name)? {
        ParameterValue::String(s) => Ok(s),
        other => bail!("parameter {name} must be string, got {}", other.type_name()),
    }
}

fn require_u32(node: &Node, name: &str) -> Result<u32> {
    let v = require_i64(node, name)?;
    if v < 0 || v > i64::from(u32::MAX) {
        bail!("parameter {name} out of u32 range: {v}");
    }
    Ok(v as u32)
}

fn require_u16(node: &Node, name: &str) -> Result<u16> {
    let v = require_i64(node, name)?;
    if v < 0 || v > i64::from(u16::MAX) {
        bail!("parameter {name} out of u16 range: {v}");
    }
    Ok(v as u16)
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
    fn defaults_shape() {
        // Smoke: config fields are public and constructible for tests.
        let cfg = CaptureConfig {
            output_topic: "/audio/mic".into(),
            device: String::new(),
            sample_rate: 16_000,
            channels: 1,
            chunk_ms: 20,
        };
        assert_eq!(cfg.sample_rate, 16_000);
        assert_eq!(cfg.channels, 1);
    }
}
