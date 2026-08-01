//! Node parameters for the USB camera capture tool node.

use anyhow::{bail, Context, Result};
use crate::{Node, ParameterValue};

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub output_topic: String,
    /// Empty → first camera (index 0). Otherwise numeric index or exact device name.
    pub device: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frame_id: String,
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
        let width = require_u32(node, "width")?;
        let height = require_u32(node, "height")?;
        let fps = require_u32(node, "fps")?;
        let frame_id = require_string(node, "frame_id")?;

        if output_topic.is_empty() {
            bail!("output_topic must be non-empty");
        }
        if width == 0 || height == 0 {
            bail!("width and height must be > 0");
        }
        if fps == 0 {
            bail!("fps must be > 0");
        }

        Ok(Self {
            output_topic,
            device,
            width,
            height,
            fps,
            frame_id,
        })
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter(
        "output_topic",
        ParameterValue::String("/camera/image_raw".into()),
    )?;
    node.declare_parameter("device", ParameterValue::String(String::new()))?;
    node.declare_parameter("width", ParameterValue::Integer(640))?;
    node.declare_parameter("height", ParameterValue::Integer(480))?;
    node.declare_parameter("fps", ParameterValue::Integer(30))?;
    node.declare_parameter("frame_id", ParameterValue::String("camera".into()))?;
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
        let cfg = CaptureConfig {
            output_topic: "/camera/image_raw".into(),
            device: String::new(),
            width: 640,
            height: 480,
            fps: 30,
            frame_id: "camera".into(),
        };
        assert_eq!(cfg.width, 640);
        assert_eq!(cfg.fps, 30);
    }
}
