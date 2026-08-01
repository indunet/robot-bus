//! Node parameters for the EtherCAT joint tool node.
//!
//! Scalar keys go through the Node parameter store. The `joints` list is nested
//! YAML and is parsed separately (parameter MVP is scalars-only).

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use crate::{Node, ParameterValue};

/// CiA402 cyclic synchronous modes supported by this node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JointMode {
    Csp,
    Csv,
    Cst,
}

impl JointMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csp => "csp",
            Self::Csv => "csv",
            Self::Cst => "cst",
        }
    }

    /// CiA402 Modes of Operation value (object 0x6060).
    pub fn modes_of_operation(self) -> i8 {
        match self {
            Self::Csp => 8,
            Self::Csv => 9,
            Self::Cst => 10,
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "csp" => Ok(Self::Csp),
            "csv" => Ok(Self::Csv),
            "cst" => Ok(Self::Cst),
            other => bail!("unknown joint mode {other:?}; expected csp|csv|cst"),
        }
    }
}

/// PDI byte offsets for a minimal CiA402 process image layout.
#[derive(Debug, Clone, Deserialize)]
pub struct PdoOffsets {
    #[serde(default = "default_controlword")]
    pub controlword: usize,
    #[serde(default = "default_target")]
    pub target: usize,
    #[serde(default = "default_statusword")]
    pub statusword: usize,
    #[serde(default = "default_actual")]
    pub actual: usize,
}

fn default_controlword() -> usize {
    0
}
fn default_target() -> usize {
    2
}
fn default_statusword() -> usize {
    0
}
fn default_actual() -> usize {
    2
}

impl Default for PdoOffsets {
    fn default() -> Self {
        Self {
            controlword: default_controlword(),
            target: default_target(),
            statusword: default_statusword(),
            actual: default_actual(),
        }
    }
}

/// One configured joint / drive axis.
#[derive(Debug, Clone, Deserialize)]
pub struct JointConfig {
    pub name: String,
    pub station_address: u16,
    pub mode: JointMode,
    #[serde(default = "default_ticks")]
    pub encoder_ticks_per_rev: f64,
    #[serde(default = "default_gear")]
    pub gear_ratio: f64,
    #[serde(default = "default_direction")]
    pub direction: i32,
    #[serde(default)]
    pub position_offset_rad: f64,
    #[serde(default)]
    pub pdo: PdoOffsets,
}

fn default_ticks() -> f64 {
    524_288.0
}
fn default_gear() -> f64 {
    1.0
}
fn default_direction() -> i32 {
    1
}

/// Which master implementation to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Ethercrab,
    Mock,
}

impl BackendKind {
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ethercrab" => Ok(Self::Ethercrab),
            "mock" => Ok(Self::Mock),
            other => bail!("unknown backend {other:?}; expected ethercrab|mock"),
        }
    }
}

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct EthercatJointConfig {
    pub iface: String,
    pub backend: BackendKind,
    pub cycle_ns: u64,
    pub state_rate_hz: u32,
    pub command_timeout_ms: u32,
    pub auto_enable: bool,
    pub output_topic: String,
    pub command_topic: String,
    pub diagnostics_topic: String,
    pub enable_service: String,
    pub fault_reset_service: String,
    pub frame_id: String,
    pub joints: Vec<JointConfig>,
}

impl EthercatJointConfig {
    /// Declare scalar defaults, overlay YAML (scalars + joints), then read back.
    pub fn load(node: &mut Node, params_path: Option<&str>) -> Result<Self> {
        declare_defaults(node)?;
        let mut joints = Vec::new();
        if let Some(path) = params_path {
            joints = load_yaml_into_node(node, path)
                .with_context(|| format!("load parameters from {path}"))?;
        }
        let mut cfg = Self::from_node(node)?;
        if !joints.is_empty() {
            cfg.joints = joints;
        }
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn from_node(node: &Node) -> Result<Self> {
        Ok(Self {
            iface: require_string(node, "iface")?,
            backend: BackendKind::parse(&require_string(node, "backend")?)?,
            cycle_ns: require_u64(node, "cycle_ns")?,
            state_rate_hz: require_u32(node, "state_rate_hz")?,
            command_timeout_ms: require_u32(node, "command_timeout_ms")?,
            auto_enable: require_bool(node, "auto_enable")?,
            output_topic: require_string(node, "output_topic")?,
            command_topic: require_string(node, "command_topic")?,
            diagnostics_topic: require_string(node, "diagnostics_topic")?,
            enable_service: require_string(node, "enable_service")?,
            fault_reset_service: require_string(node, "fault_reset_service")?,
            frame_id: require_string(node, "frame_id")?,
            joints: Vec::new(),
        })
    }

    fn validate(&self) -> Result<()> {
        if self.output_topic.is_empty() || self.command_topic.is_empty() {
            bail!("output_topic and command_topic must be non-empty");
        }
        if self.cycle_ns == 0 {
            bail!("cycle_ns must be > 0");
        }
        if self.state_rate_hz == 0 {
            bail!("state_rate_hz must be > 0");
        }
        if self.joints.is_empty() {
            bail!("at least one joint must be configured under `joints:`");
        }
        let mut names = std::collections::HashSet::new();
        for j in &self.joints {
            if j.name.is_empty() {
                bail!("joint name must be non-empty");
            }
            if !names.insert(j.name.clone()) {
                bail!("duplicate joint name {}", j.name);
            }
            if j.encoder_ticks_per_rev <= 0.0 {
                bail!("joint {}: encoder_ticks_per_rev must be > 0", j.name);
            }
            if j.gear_ratio == 0.0 {
                bail!("joint {}: gear_ratio must be non-zero", j.name);
            }
            if j.direction != 1 && j.direction != -1 {
                bail!("joint {}: direction must be 1 or -1", j.name);
            }
        }
        Ok(())
    }
}

fn declare_defaults(node: &mut Node) -> Result<()> {
    node.declare_parameter("iface", ParameterValue::String("eth0".into()))?;
    node.declare_parameter("backend", ParameterValue::String("ethercrab".into()))?;
    node.declare_parameter("cycle_ns", ParameterValue::Integer(1_000_000))?;
    node.declare_parameter("state_rate_hz", ParameterValue::Integer(100))?;
    node.declare_parameter("command_timeout_ms", ParameterValue::Integer(100))?;
    node.declare_parameter("auto_enable", ParameterValue::Bool(true))?;
    node.declare_parameter(
        "output_topic",
        ParameterValue::String("/joint_states".into()),
    )?;
    node.declare_parameter(
        "command_topic",
        ParameterValue::String("/joint_commands".into()),
    )?;
    node.declare_parameter(
        "diagnostics_topic",
        ParameterValue::String("/diagnostics".into()),
    )?;
    node.declare_parameter(
        "enable_service",
        ParameterValue::String("/ethercat_joint/enable".into()),
    )?;
    node.declare_parameter(
        "fault_reset_service",
        ParameterValue::String("/ethercat_joint/fault_reset".into()),
    )?;
    node.declare_parameter(
        "frame_id",
        ParameterValue::String("ethercat_joint".into()),
    )?;
    Ok(())
}

/// Apply scalar YAML params to the node; return parsed `joints` list.
fn load_yaml_into_node(node: &mut Node, path: impl AsRef<Path>) -> Result<Vec<JointConfig>> {
    let text = fs::read_to_string(path.as_ref())
        .with_context(|| format!("read {}", path.as_ref().display()))?;
    let root: serde_yaml::Value =
        serde_yaml::from_str(&text).context("parse ethercat joint YAML")?;
    let mut mapping = extract_param_mapping(root)?;

    let joints = if let Some(value) = mapping.remove(serde_yaml::Value::String("joints".into())) {
        serde_yaml::from_value::<Vec<JointConfig>>(value).context("parse joints list")?
    } else {
        Vec::new()
    };

    let filtered = serde_yaml::to_string(&serde_yaml::Value::Mapping(mapping))
        .context("serialize filtered parameters")?;
    node.load_parameters_from_yaml_str(&filtered)
        .context("load scalar parameters")?;
    Ok(joints)
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

fn require_bool(node: &Node, name: &str) -> Result<bool> {
    match node.get_parameter(name)? {
        ParameterValue::Bool(b) => Ok(b),
        other => bail!("parameter {name} must be bool, got {}", other.type_name()),
    }
}

fn require_u32(node: &Node, name: &str) -> Result<u32> {
    let v = require_i64(node, name)?;
    if v < 0 || v > i64::from(u32::MAX) {
        bail!("parameter {name} out of u32 range: {v}");
    }
    Ok(v as u32)
}

fn require_u64(node: &Node, name: &str) -> Result<u64> {
    let v = require_i64(node, name)?;
    if v < 0 {
        bail!("parameter {name} must be >= 0");
    }
    Ok(v as u64)
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
    use crate::NodeOptions;
    use std::path::PathBuf;

    #[test]
    fn parse_example_yaml() {
        let mut node = Node::with_options("cfg_test", NodeOptions::inproc());
        let dir = tempfile_dir();
        let path = dir.join("ec.yaml");
        std::fs::write(&path, crate::ethercat_joint::EXAMPLE_CONFIG).unwrap();
        let cfg = EthercatJointConfig::load(&mut node, Some(path.to_str().unwrap())).unwrap();
        assert_eq!(cfg.joints.len(), 1);
        assert_eq!(cfg.joints[0].name, "joint_1");
        assert_eq!(cfg.joints[0].mode, JointMode::Csp);
        assert_eq!(cfg.backend, BackendKind::Ethercrab);
        assert_eq!(cfg.state_rate_hz, 100);
    }

    fn tempfile_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("robot_bus_ec_cfg_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn mode_ops() {
        assert_eq!(JointMode::Csp.modes_of_operation(), 8);
        assert_eq!(JointMode::Csv.modes_of_operation(), 9);
        assert_eq!(JointMode::Cst.modes_of_operation(), 10);
    }
}
