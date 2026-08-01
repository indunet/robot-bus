//! Node parameters for robot_state_publisher.

use anyhow::{bail, Context, Result};
use crate::{Node, ParameterValue};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RobotStatePublisherConfig {
    pub urdf_file: PathBuf,
    pub joint_states_topic: String,
    pub tf_topic: String,
    pub tf_static_topic: String,
    pub static_publish_rate_hz: f64,
    /// `None` → use URDF `limit.lower` (or 0.0) when a joint is missing.
    pub missing_joint_position: Option<f64>,
}

impl RobotStatePublisherConfig {
    pub fn load(node: &mut Node, params_path: Option<&str>) -> Result<Self> {
        declare_defaults(node)?;
        let mut missing_joint_position = None;
        if let Some(path) = params_path {
            missing_joint_position = load_yaml_into_node(node, path)
                .with_context(|| format!("load parameters from {path}"))?;
        }
        let mut cfg = Self::from_node(node)?;
        cfg.missing_joint_position = missing_joint_position;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn from_node(node: &Node) -> Result<Self> {
        Ok(Self {
            urdf_file: PathBuf::from(require_string(node, "urdf_file")?),
            joint_states_topic: require_string(node, "joint_states_topic")?,
            tf_topic: require_string(node, "tf_topic")?,
            tf_static_topic: require_string(node, "tf_static_topic")?,
            static_publish_rate_hz: require_f64(node, "static_publish_rate_hz")?,
            missing_joint_position: None,
        })
    }

    fn validate(&self) -> Result<()> {
        if self.urdf_file.as_os_str().is_empty() {
            bail!("urdf_file must be non-empty");
        }
        if !self.urdf_file.exists() {
            bail!("urdf_file not found: {}", self.urdf_file.display());
        }
        if self.joint_states_topic.is_empty()
            || self.tf_topic.is_empty()
            || self.tf_static_topic.is_empty()
        {
            bail!("joint_states_topic, tf_topic, and tf_static_topic must be non-empty");
        }
        if self.static_publish_rate_hz < 0.0 {
            bail!("static_publish_rate_hz must be >= 0");
        }
        Ok(())
    }

    pub fn urdf_path(&self) -> &Path {
        &self.urdf_file
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter("urdf_file", ParameterValue::String(String::new()))?;
    node.declare_parameter(
        "joint_states_topic",
        ParameterValue::String("/joint_states".into()),
    )?;
    node.declare_parameter("tf_topic", ParameterValue::String("/tf".into()))?;
    node.declare_parameter(
        "tf_static_topic",
        ParameterValue::String("/tf_static".into()),
    )?;
    node.declare_parameter("static_publish_rate_hz", ParameterValue::Double(1.0))?;
    Ok(())
}

/// Apply scalar YAML params; return optional `missing_joint_position`.
fn load_yaml_into_node(node: &mut Node, path: impl AsRef<Path>) -> Result<Option<f64>> {
    let text = fs::read_to_string(path.as_ref())
        .with_context(|| format!("read {}", path.as_ref().display()))?;
    let root: serde_yaml::Value =
        serde_yaml::from_str(&text).context("parse robot_state_publisher YAML")?;
    let mut mapping = extract_param_mapping(root)?;

    let missing = if let Some(value) =
        mapping.remove(serde_yaml::Value::String("missing_joint_position".into()))
    {
        match value {
            serde_yaml::Value::Number(n) => Some(
                n.as_f64()
                    .ok_or_else(|| anyhow::anyhow!("missing_joint_position must be a number"))?,
            ),
            serde_yaml::Value::Null => None,
            other => bail!("missing_joint_position must be a number, got {other:?}"),
        }
    } else {
        None
    };

    let filtered = serde_yaml::to_string(&serde_yaml::Value::Mapping(mapping))
        .context("serialize filtered parameters")?;
    node.load_parameters_from_yaml_str(&filtered)
        .context("load scalar parameters")?;
    Ok(missing)
}

fn extract_param_mapping(root: serde_yaml::Value) -> Result<serde_yaml::Mapping> {
    let mapping = match root {
        serde_yaml::Value::Mapping(m) => m,
        serde_yaml::Value::Null => return Ok(serde_yaml::Mapping::new()),
        other => bail!("expected a YAML mapping at root, got {other:?}"),
    };

    if let Some(serde_yaml::Value::Mapping(m)) =
        mapping.get(serde_yaml::Value::String("ros__parameters".into()))
    {
        return Ok(m.clone());
    }
    if let Some(serde_yaml::Value::Mapping(ns)) =
        mapping.get(serde_yaml::Value::String("/**".into()))
    {
        if let Some(serde_yaml::Value::Mapping(m)) =
            ns.get(serde_yaml::Value::String("ros__parameters".into()))
        {
            return Ok(m.clone());
        }
    }
    Ok(mapping)
}

fn require_string(node: &Node, name: &str) -> Result<String> {
    match node.get_parameter(name)? {
        ParameterValue::String(s) => Ok(s),
        other => bail!("parameter {name} must be string, got {}", other.type_name()),
    }
}

fn require_f64(node: &Node, name: &str) -> Result<f64> {
    match node.get_parameter(name)? {
        ParameterValue::Double(d) => Ok(d),
        ParameterValue::Integer(i) => Ok(i as f64),
        other => bail!("parameter {name} must be number, got {}", other.type_name()),
    }
}
