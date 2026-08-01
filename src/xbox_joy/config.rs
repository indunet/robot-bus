//! Node parameters for the Xbox joy tool node.

use anyhow::{bail, Context, Result};
use crate::{Node, ParameterValue};

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct JoyConfig {
    pub output_topic: String,
    pub rumble_topic: String,
    /// Empty → first connected pad. Otherwise numeric index or name substring.
    pub device: String,
    pub rate_hz: u32,
    pub frame_id: String,
    pub deadzone: f32,
}

impl JoyConfig {
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
        let rumble_topic = require_string(node, "rumble_topic")?;
        let device = require_string(node, "device")?;
        let rate_hz = require_u32(node, "rate_hz")?;
        let frame_id = require_string(node, "frame_id")?;
        let deadzone = require_f32(node, "deadzone")?;

        if output_topic.is_empty() {
            bail!("output_topic must be non-empty");
        }
        if rumble_topic.is_empty() {
            bail!("rumble_topic must be non-empty");
        }
        if rate_hz == 0 {
            bail!("rate_hz must be > 0");
        }
        if !(0.0..1.0).contains(&deadzone) {
            bail!("deadzone must be in [0.0, 1.0), got {deadzone}");
        }

        Ok(Self {
            output_topic,
            rumble_topic,
            device,
            rate_hz,
            frame_id,
            deadzone,
        })
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter(
        "output_topic",
        ParameterValue::String("/xbox_joy".into()),
    )?;
    node.declare_parameter(
        "rumble_topic",
        ParameterValue::String("/xbox_joy/rumble".into()),
    )?;
    node.declare_parameter("device", ParameterValue::String(String::new()))?;
    node.declare_parameter("rate_hz", ParameterValue::Integer(50))?;
    node.declare_parameter(
        "frame_id",
        ParameterValue::String("xbox_joy".into()),
    )?;
    node.declare_parameter("deadzone", ParameterValue::Double(0.1))?;
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

fn require_f32(node: &Node, name: &str) -> Result<f32> {
    match node.get_parameter(name)? {
        ParameterValue::Double(d) => Ok(d as f32),
        ParameterValue::Integer(i) => Ok(i as f32),
        other => bail!(
            "parameter {name} must be number, got {}",
            other.type_name()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_shape() {
        let cfg = JoyConfig {
            output_topic: "/xbox_joy".into(),
            rumble_topic: "/xbox_joy/rumble".into(),
            device: String::new(),
            rate_hz: 50,
            frame_id: "xbox_joy".into(),
            deadzone: 0.1,
        };
        assert_eq!(cfg.rate_hz, 50);
        assert!((cfg.deadzone - 0.1).abs() < f32::EPSILON);
    }
}
