//! Node parameters for the AprilTag detector.

use anyhow::{bail, Context, Result};

use crate::{Node, ParameterValue};

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    pub input_topic: String,
    pub output_topic: String,
    pub family: String,
    pub bits_corrected: i64,
    pub decimate: f64,
    pub blur: f64,
    pub refine_edges: bool,
    pub sharpening: f64,
    pub threads: i64,
}

impl DetectorConfig {
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
        let family = require_string(node, "family")?;
        let bits_corrected = require_i64(node, "bits_corrected")?;
        let decimate = require_f64(node, "decimate")?;
        let blur = require_f64(node, "blur")?;
        let refine_edges = require_bool(node, "refine_edges")?;
        let sharpening = require_f64(node, "sharpening")?;
        let threads = require_i64(node, "threads")?;

        if input_topic.is_empty() || output_topic.is_empty() {
            bail!("input_topic and output_topic must be non-empty");
        }
        if family.is_empty() {
            bail!("family must be non-empty");
        }
        if bits_corrected < 0 {
            bail!("bits_corrected must be >= 0");
        }
        if !(decimate > 0.0) {
            bail!("decimate must be > 0");
        }
        if blur < 0.0 {
            bail!("blur must be >= 0");
        }
        if sharpening < 0.0 {
            bail!("sharpening must be >= 0");
        }
        if threads < 1 || threads > 255 {
            bail!("threads must be in 1..=255");
        }

        Ok(Self {
            input_topic,
            output_topic,
            family,
            bits_corrected,
            decimate,
            blur,
            refine_edges,
            sharpening,
            threads,
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
        ParameterValue::String("/apriltag/detections".into()),
    )?;
    node.declare_parameter("family", ParameterValue::String("tag36h11".into()))?;
    node.declare_parameter("bits_corrected", ParameterValue::Integer(2))?;
    node.declare_parameter("decimate", ParameterValue::Double(2.0))?;
    node.declare_parameter("blur", ParameterValue::Double(0.0))?;
    node.declare_parameter("refine_edges", ParameterValue::Bool(true))?;
    node.declare_parameter("sharpening", ParameterValue::Double(0.25))?;
    node.declare_parameter("threads", ParameterValue::Integer(1))?;
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

fn require_f64(node: &Node, name: &str) -> Result<f64> {
    match node.get_parameter(name)? {
        ParameterValue::Double(d) => Ok(d),
        ParameterValue::Integer(i) => Ok(i as f64),
        other => bail!(
            "parameter {name} must be number, got {}",
            other.type_name()
        ),
    }
}

fn require_bool(node: &Node, name: &str) -> Result<bool> {
    match node.get_parameter(name)? {
        ParameterValue::Bool(b) => Ok(b),
        other => bail!("parameter {name} must be bool, got {}", other.type_name()),
    }
}
