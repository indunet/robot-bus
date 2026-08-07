//! Local node parameters (ROS 2–style declare / get / set / YAML load).

use std::collections::HashMap;
use std::path::Path;

use crate::errors::{BusError, Result};

/// Scalar parameter value (MVP: no arrays or nested types).
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    Bool(bool),
    Integer(i64),
    Double(f64),
    String(String),
}

impl ParameterValue {
    /// Discriminant name for error messages (`bool`, `integer`, …).
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Double(_) => "double",
            Self::String(_) => "string",
        }
    }

    /// Whether `other` has the same variant (ignoring payload).
    pub fn same_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Named parameter snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: ParameterValue,
}

/// Per-node parameter table.
#[derive(Debug, Default, Clone)]
pub(crate) struct ParameterStore {
    values: HashMap<String, ParameterValue>,
}

impl ParameterStore {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn declare(&mut self, name: impl Into<String>, value: ParameterValue) -> Result<()> {
        let name = name.into();
        if self.values.contains_key(&name) {
            return Err(BusError::ParameterAlreadyDeclared { name });
        }
        self.values.insert(name, value);
        Ok(())
    }

    pub(crate) fn get(&self, name: &str) -> Result<ParameterValue> {
        self.values
            .get(name)
            .cloned()
            .ok_or_else(|| BusError::ParameterNotDeclared {
                name: name.to_string(),
            })
    }

    pub(crate) fn set(&mut self, name: &str, value: ParameterValue) -> Result<()> {
        let existing = self
            .values
            .get(name)
            .ok_or_else(|| BusError::ParameterNotDeclared {
                name: name.to_string(),
            })?;
        let value = match coerce_compatible(existing, value) {
            Ok(v) => v,
            Err((expected, got)) => {
                return Err(BusError::ParameterTypeMismatch {
                    name: name.to_string(),
                    expected,
                    got,
                });
            }
        };
        self.values.insert(name.to_string(), value);
        Ok(())
    }

    /// Declare if missing; otherwise set (type must match).
    pub(crate) fn load(&mut self, name: impl Into<String>, value: ParameterValue) -> Result<()> {
        let name = name.into();
        if self.values.contains_key(&name) {
            self.set(&name, value)
        } else {
            self.declare(name, value)
        }
    }

    pub(crate) fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub(crate) fn list(&self) -> Vec<Parameter> {
        let mut out: Vec<Parameter> = self
            .values
            .iter()
            .map(|(name, value)| Parameter {
                name: name.clone(),
                value: value.clone(),
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    /// Load scalar parameters from a YAML document.
    ///
    /// Supported shapes:
    /// - flat map: `{ max_speed: 1.5, frame_id: base_link }`
    /// - ROS 2 style: `{ ros__parameters: { … } }`
    /// - wildcard: `{ "/**": { ros__parameters: { … } } }`
    ///
    /// Undeclared names are declared; existing names are updated (`set`).
    pub(crate) fn load_from_yaml_str(&mut self, yaml: &str) -> Result<()> {
        let root: serde_yaml::Value =
            serde_yaml::from_str(yaml).map_err(|e| BusError::ParameterYaml(e.to_string()))?;
        let mapping = extract_param_mapping(root)?;
        for (key, value) in mapping {
            let name = key
                .as_str()
                .ok_or_else(|| BusError::ParameterYaml("parameter name must be a string".into()))?
                .to_string();
            let param = yaml_to_parameter_value(&value)?;
            self.load(name, param)?;
        }
        Ok(())
    }

    pub(crate) fn load_from_yaml_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| {
            BusError::ParameterYaml(format!("failed to read {}: {e}", path.display()))
        })?;
        self.load_from_yaml_str(&text)
    }
}

fn extract_param_mapping(root: serde_yaml::Value) -> Result<serde_yaml::Mapping> {
    let mapping = match root {
        serde_yaml::Value::Mapping(m) => m,
        serde_yaml::Value::Null => return Ok(serde_yaml::Mapping::new()),
        other => {
            return Err(BusError::ParameterYaml(format!(
                "expected a YAML mapping at root, got {other:?}"
            )));
        }
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

fn yaml_to_parameter_value(value: &serde_yaml::Value) -> Result<ParameterValue> {
    match value {
        serde_yaml::Value::Bool(b) => Ok(ParameterValue::Bool(*b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(ParameterValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(ParameterValue::Double(f))
            } else {
                Err(BusError::ParameterYaml(format!(
                    "unsupported number: {n:?}"
                )))
            }
        }
        serde_yaml::Value::String(s) => Ok(ParameterValue::String(s.clone())),
        other => Err(BusError::ParameterYaml(format!(
            "unsupported parameter value (scalars only): {other:?}"
        ))),
    }
}

/// Allow Integer ↔ Double when the value fits (JS/YAML whole numbers).
fn coerce_compatible(
    existing: &ParameterValue,
    value: ParameterValue,
) -> std::result::Result<ParameterValue, (&'static str, &'static str)> {
    if existing.same_type(&value) {
        return Ok(value);
    }
    match (existing, value) {
        (ParameterValue::Double(_), ParameterValue::Integer(i)) => {
            Ok(ParameterValue::Double(i as f64))
        }
        (ParameterValue::Integer(_), ParameterValue::Double(d)) if d.fract() == 0.0 => {
            Ok(ParameterValue::Integer(d as i64))
        }
        (_, value) => Err((existing.type_name(), value.type_name())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declare_get_set_list() {
        let mut store = ParameterStore::new();
        store
            .declare("max_speed", ParameterValue::Double(1.5))
            .unwrap();
        store
            .declare("frame_id", ParameterValue::String("base_link".into()))
            .unwrap();

        assert_eq!(store.get("max_speed").unwrap(), ParameterValue::Double(1.5));
        store.set("max_speed", ParameterValue::Double(2.0)).unwrap();
        assert_eq!(store.get("max_speed").unwrap(), ParameterValue::Double(2.0));

        let names: Vec<_> = store.list().into_iter().map(|p| p.name).collect();
        assert_eq!(names, vec!["frame_id", "max_speed"]);
        assert!(store.has("frame_id"));
        assert!(!store.has("missing"));
    }

    #[test]
    fn reject_duplicate_undeclared_and_type_mismatch() {
        let mut store = ParameterStore::new();
        store.declare("flag", ParameterValue::Bool(true)).unwrap();

        assert!(matches!(
            store.declare("flag", ParameterValue::Bool(false)),
            Err(BusError::ParameterAlreadyDeclared { .. })
        ));
        assert!(matches!(
            store.get("nope"),
            Err(BusError::ParameterNotDeclared { .. })
        ));
        assert!(matches!(
            store.set("flag", ParameterValue::Integer(1)),
            Err(BusError::ParameterTypeMismatch { .. })
        ));
        assert!(matches!(
            store.set("nope", ParameterValue::Bool(false)),
            Err(BusError::ParameterNotDeclared { .. })
        ));
    }

    #[test]
    fn load_flat_yaml() {
        let mut store = ParameterStore::new();
        store
            .load_from_yaml_str(
                r#"
max_speed: 1.5
frame_id: base_link
enabled: true
count: 3
"#,
            )
            .unwrap();
        assert_eq!(store.get("max_speed").unwrap(), ParameterValue::Double(1.5));
        assert_eq!(
            store.get("frame_id").unwrap(),
            ParameterValue::String("base_link".into())
        );
        assert_eq!(store.get("enabled").unwrap(), ParameterValue::Bool(true));
        assert_eq!(store.get("count").unwrap(), ParameterValue::Integer(3));
    }

    #[test]
    fn load_ros_parameters_yaml_and_override() {
        let mut store = ParameterStore::new();
        store
            .declare("max_speed", ParameterValue::Double(1.0))
            .unwrap();
        store
            .load_from_yaml_str(
                r#"
ros__parameters:
  max_speed: 2.5
  frame_id: map
"#,
            )
            .unwrap();
        assert_eq!(store.get("max_speed").unwrap(), ParameterValue::Double(2.5));
        assert_eq!(
            store.get("frame_id").unwrap(),
            ParameterValue::String("map".into())
        );
    }

    #[test]
    fn load_wildcard_ros_parameters_yaml() {
        let mut store = ParameterStore::new();
        store
            .load_from_yaml_str(
                r#"
"/**":
  ros__parameters:
    enabled: false
"#,
            )
            .unwrap();
        assert_eq!(store.get("enabled").unwrap(), ParameterValue::Bool(false));
    }

    #[test]
    fn load_yaml_rejects_nested_and_type_mismatch() {
        let mut store = ParameterStore::new();
        store.declare("flag", ParameterValue::Bool(true)).unwrap();
        assert!(matches!(
            store.load_from_yaml_str("nested:\n  a: 1\n"),
            Err(BusError::ParameterYaml(_))
        ));
        assert!(matches!(
            store.load_from_yaml_str("flag: 1\n"),
            Err(BusError::ParameterTypeMismatch { .. })
        ));
    }

    #[test]
    fn load_yaml_file() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("robot_bus_params_{}.yaml", std::process::id()));
        std::fs::write(&path, "max_speed: 9.0\nframe_id: odom\n").unwrap();
        let mut store = ParameterStore::new();
        store.load_from_yaml_file(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(store.get("max_speed").unwrap(), ParameterValue::Double(9.0));
        assert_eq!(
            store.get("frame_id").unwrap(),
            ParameterValue::String("odom".into())
        );
    }

    #[test]
    fn integer_double_coercion_on_set() {
        let mut store = ParameterStore::new();
        store
            .declare("max_speed", ParameterValue::Double(1.5))
            .unwrap();
        store.set("max_speed", ParameterValue::Integer(2)).unwrap();
        assert_eq!(store.get("max_speed").unwrap(), ParameterValue::Double(2.0));

        store.declare("count", ParameterValue::Integer(3)).unwrap();
        store.set("count", ParameterValue::Double(9.0)).unwrap();
        assert_eq!(store.get("count").unwrap(), ParameterValue::Integer(9));
        assert!(matches!(
            store.set("count", ParameterValue::Double(1.5)),
            Err(BusError::ParameterTypeMismatch { .. })
        ));
    }
}
