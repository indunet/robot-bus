//! Node parameters for the static transform publisher.

use anyhow::{bail, Context, Result};
use crate::tf::RigidTransform;
use crate::{Node, ParameterValue};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Xyz {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rpy {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Xyzw {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransformEntry {
    pub parent_frame_id: String,
    pub child_frame_id: String,
    pub translation: Xyz,
    #[serde(default)]
    pub rotation_rpy: Option<Rpy>,
    #[serde(default)]
    pub rotation_xyzw: Option<Xyzw>,
}

impl TransformEntry {
    pub fn to_rigid(&self) -> Result<RigidTransform> {
        match (&self.rotation_rpy, &self.rotation_xyzw) {
            (Some(rpy), None) => Ok(RigidTransform::from_xyz_rpy(
                self.translation.x,
                self.translation.y,
                self.translation.z,
                rpy.roll,
                rpy.pitch,
                rpy.yaw,
            )),
            (None, Some(q)) => Ok(RigidTransform::from_xyz_xyzw(
                self.translation.x,
                self.translation.y,
                self.translation.z,
                q.x,
                q.y,
                q.z,
                q.w,
            )),
            (None, None) => Ok(RigidTransform::from_xyz_rpy(
                self.translation.x,
                self.translation.y,
                self.translation.z,
                0.0,
                0.0,
                0.0,
            )),
            (Some(_), Some(_)) => bail!(
                "transform {} → {}: specify only one of rotation_rpy or rotation_xyzw",
                self.parent_frame_id,
                self.child_frame_id
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaticTransformConfig {
    pub output_topic: String,
    pub publish_rate_hz: f64,
    pub transforms: Vec<TransformEntry>,
}

impl StaticTransformConfig {
    pub fn load(node: &mut Node, params_path: Option<&str>) -> Result<Self> {
        declare_defaults(node)?;
        let mut transforms = Vec::new();
        if let Some(path) = params_path {
            transforms = load_yaml_into_node(node, path)
                .with_context(|| format!("load parameters from {path}"))?;
        }
        let mut cfg = Self::from_node(node)?;
        if !transforms.is_empty() {
            cfg.transforms = transforms;
        }
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn from_node(node: &Node) -> Result<Self> {
        Ok(Self {
            output_topic: require_string(node, "output_topic")?,
            publish_rate_hz: require_f64(node, "publish_rate_hz")?,
            transforms: Vec::new(),
        })
    }

    fn validate(&self) -> Result<()> {
        if self.output_topic.is_empty() {
            bail!("output_topic must be non-empty");
        }
        if self.publish_rate_hz < 0.0 {
            bail!("publish_rate_hz must be >= 0");
        }
        if self.transforms.is_empty() {
            bail!("at least one transform must be configured under `transforms:`");
        }
        let mut children = std::collections::HashSet::new();
        for t in &self.transforms {
            if t.parent_frame_id.is_empty() || t.child_frame_id.is_empty() {
                bail!("parent_frame_id and child_frame_id must be non-empty");
            }
            if t.parent_frame_id == t.child_frame_id {
                bail!(
                    "parent and child must differ ({})",
                    t.parent_frame_id
                );
            }
            if !children.insert(t.child_frame_id.clone()) {
                bail!("duplicate child_frame_id {}", t.child_frame_id);
            }
            t.to_rigid()?;
        }
        Ok(())
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter(
        "output_topic",
        ParameterValue::String("/tf_static".into()),
    )?;
    node.declare_parameter("publish_rate_hz", ParameterValue::Double(1.0))?;
    Ok(())
}

fn load_yaml_into_node(node: &mut Node, path: impl AsRef<Path>) -> Result<Vec<TransformEntry>> {
    let text = fs::read_to_string(path.as_ref())
        .with_context(|| format!("read {}", path.as_ref().display()))?;
    let root: serde_yaml::Value =
        serde_yaml::from_str(&text).context("parse static transform YAML")?;
    let mut mapping = extract_param_mapping(root)?;

    let transforms = if let Some(value) =
        mapping.remove(serde_yaml::Value::String("transforms".into()))
    {
        serde_yaml::from_value::<Vec<TransformEntry>>(value).context("parse transforms list")?
    } else {
        Vec::new()
    };

    let filtered = serde_yaml::to_string(&serde_yaml::Value::Mapping(mapping))
        .context("serialize filtered parameters")?;
    node.load_parameters_from_yaml_str(&filtered)
        .context("load scalar parameters")?;
    Ok(transforms)
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
