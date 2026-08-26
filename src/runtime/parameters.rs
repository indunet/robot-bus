//! Local node parameters (ROS 2–style declare / get / set / list / YAML load).

use std::collections::{BTreeSet, HashMap};
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

    /// ROS 2–style `as_bool()`.
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(v) => Ok(*v),
            other => Err(type_mismatch("", "bool", other.type_name())),
        }
    }

    /// ROS 2–style `as_int()` / integer accessor.
    pub fn as_int(&self) -> Result<i64> {
        match self {
            Self::Integer(v) => Ok(*v),
            other => Err(type_mismatch("", "integer", other.type_name())),
        }
    }

    /// ROS 2–style `as_double()`.
    pub fn as_double(&self) -> Result<f64> {
        match self {
            Self::Double(v) => Ok(*v),
            other => Err(type_mismatch("", "double", other.type_name())),
        }
    }

    /// ROS 2–style `as_string()`.
    pub fn as_string(&self) -> Result<&str> {
        match self {
            Self::String(v) => Ok(v.as_str()),
            other => Err(type_mismatch("", "string", other.type_name())),
        }
    }
}

impl From<bool> for ParameterValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i64> for ParameterValue {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<i32> for ParameterValue {
    fn from(v: i32) -> Self {
        Self::Integer(i64::from(v))
    }
}

impl From<f64> for ParameterValue {
    fn from(v: f64) -> Self {
        Self::Double(v)
    }
}

impl From<f32> for ParameterValue {
    fn from(v: f32) -> Self {
        Self::Double(f64::from(v))
    }
}

impl From<String> for ParameterValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for ParameterValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

/// Named parameter snapshot (ROS 2 `rclcpp::Parameter` / `rclpy.Parameter`).
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: ParameterValue,
}

impl Parameter {
    pub fn new(name: impl Into<String>, value: impl Into<ParameterValue>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match &self.value {
            ParameterValue::Bool(v) => Ok(*v),
            other => Err(type_mismatch(&self.name, "bool", other.type_name())),
        }
    }

    pub fn as_int(&self) -> Result<i64> {
        match &self.value {
            ParameterValue::Integer(v) => Ok(*v),
            other => Err(type_mismatch(&self.name, "integer", other.type_name())),
        }
    }

    pub fn as_double(&self) -> Result<f64> {
        match &self.value {
            ParameterValue::Double(v) => Ok(*v),
            other => Err(type_mismatch(&self.name, "double", other.type_name())),
        }
    }

    pub fn as_string(&self) -> Result<&str> {
        match &self.value {
            ParameterValue::String(v) => Ok(v.as_str()),
            other => Err(type_mismatch(&self.name, "string", other.type_name())),
        }
    }
}

/// Result of [`Node::list_parameters`](crate::Node::list_parameters) (ROS 2 shape).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListParametersResult {
    /// Matching parameter names.
    pub names: Vec<String>,
    /// Unique parent prefixes among matches (dot-separated).
    pub prefixes: Vec<String>,
}

/// Unlimited depth when listing (ROS 2 `DEPTH_RECURSIVE` / `0`).
pub const PARAMETER_DEPTH_RECURSIVE: u64 = 0;

fn type_mismatch(name: &str, expected: &'static str, got: &'static str) -> BusError {
    BusError::ParameterTypeMismatch {
        name: name.to_string(),
        expected,
        got,
    }
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

    pub(crate) fn declare(
        &mut self,
        name: impl Into<String>,
        value: ParameterValue,
    ) -> Result<Parameter> {
        let name = name.into();
        if self.values.contains_key(&name) {
            return Err(BusError::ParameterAlreadyDeclared { name });
        }
        self.values.insert(name.clone(), value.clone());
        Ok(Parameter { name, value })
    }

    pub(crate) fn get(&self, name: &str) -> Result<Parameter> {
        self.values
            .get(name)
            .cloned()
            .map(|value| Parameter {
                name: name.to_string(),
                value,
            })
            .ok_or_else(|| BusError::ParameterNotDeclared {
                name: name.to_string(),
            })
    }

    pub(crate) fn get_many(&self, names: &[&str]) -> Result<Vec<Parameter>> {
        names.iter().map(|name| self.get(name)).collect()
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

    pub(crate) fn set_parameter(&mut self, parameter: Parameter) -> Result<()> {
        self.set(&parameter.name, parameter.value)
    }

    pub(crate) fn set_many(
        &mut self,
        parameters: impl IntoIterator<Item = Parameter>,
    ) -> Result<()> {
        for parameter in parameters {
            self.set_parameter(parameter)?;
        }
        Ok(())
    }

    pub(crate) fn undeclare(&mut self, name: &str) -> Result<()> {
        if self.values.remove(name).is_none() {
            return Err(BusError::ParameterNotDeclared {
                name: name.to_string(),
            });
        }
        Ok(())
    }

    /// Declare if missing; otherwise set (type must match).
    pub(crate) fn load(&mut self, name: impl Into<String>, value: ParameterValue) -> Result<()> {
        let name = name.into();
        if self.values.contains_key(&name) {
            self.set(&name, value)
        } else {
            self.declare(name, value).map(|_| ())
        }
    }

    pub(crate) fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// All declared parameters with values, sorted by name.
    pub(crate) fn list_all(&self) -> Vec<Parameter> {
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

    /// ROS 2–style list by prefixes + depth (`.` hierarchy).
    ///
    /// Empty `prefixes` matches the whole tree. `depth == 0` means recursive
    /// ([`PARAMETER_DEPTH_RECURSIVE`]).
    pub(crate) fn list_parameters(&self, prefixes: &[&str], depth: u64) -> ListParametersResult {
        let mut names: Vec<String> = self
            .values
            .keys()
            .filter(|name| parameter_matches_prefixes(name, prefixes, depth))
            .cloned()
            .collect();
        names.sort();

        let mut prefix_set = BTreeSet::new();
        for name in &names {
            collect_parent_prefixes(name, &mut prefix_set);
        }

        ListParametersResult {
            names,
            prefixes: prefix_set.into_iter().collect(),
        }
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

fn parameter_matches_prefixes(name: &str, prefixes: &[&str], depth: u64) -> bool {
    if prefixes.is_empty() {
        return relative_depth_ok(name.matches('.').count() as u64 + 1, depth);
    }
    prefixes
        .iter()
        .any(|prefix| parameter_matches_prefix(name, prefix, depth))
}

fn parameter_matches_prefix(name: &str, prefix: &str, depth: u64) -> bool {
    if prefix.is_empty() {
        return relative_depth_ok(name.matches('.').count() as u64 + 1, depth);
    }
    if name == prefix {
        return true;
    }
    let dotted = format!("{prefix}.");
    match name.strip_prefix(&dotted) {
        Some(rest) => {
            let levels = rest.split('.').filter(|s| !s.is_empty()).count() as u64;
            relative_depth_ok(levels, depth)
        }
        None => false,
    }
}

fn relative_depth_ok(levels: u64, depth: u64) -> bool {
    depth == PARAMETER_DEPTH_RECURSIVE || levels <= depth
}

fn collect_parent_prefixes(name: &str, out: &mut BTreeSet<String>) {
    let mut acc = String::new();
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() <= 1 {
        return;
    }
    for (i, part) in parts.iter().enumerate().take(parts.len() - 1) {
        if i > 0 {
            acc.push('.');
        }
        acc.push_str(part);
        out.insert(acc.clone());
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

        assert_eq!(store.get("max_speed").unwrap().as_double().unwrap(), 1.5);
        store.set("max_speed", ParameterValue::Double(2.0)).unwrap();
        assert_eq!(
            store.get("max_speed").unwrap().value,
            ParameterValue::Double(2.0)
        );

        let names: Vec<_> = store.list_all().into_iter().map(|p| p.name).collect();
        assert_eq!(names, vec!["frame_id", "max_speed"]);
        assert!(store.has("frame_id"));
        assert!(!store.has("missing"));
    }

    #[test]
    fn list_parameters_prefix_and_depth() {
        let mut store = ParameterStore::new();
        for name in ["foo", "foo.bar", "foo.bar.baz", "other"] {
            store.declare(name, ParameterValue::Bool(true)).unwrap();
        }

        let all = store.list_parameters(&[], PARAMETER_DEPTH_RECURSIVE);
        assert_eq!(all.names, vec!["foo", "foo.bar", "foo.bar.baz", "other"]);
        assert!(all.prefixes.contains(&"foo".to_string()));
        assert!(all.prefixes.contains(&"foo.bar".to_string()));

        let foo = store.list_parameters(&["foo"], 1);
        assert_eq!(foo.names, vec!["foo", "foo.bar"]);

        let foo_deep = store.list_parameters(&["foo"], PARAMETER_DEPTH_RECURSIVE);
        assert_eq!(foo_deep.names, vec!["foo", "foo.bar", "foo.bar.baz"]);
    }

    #[test]
    fn parameter_accessors_and_from() {
        let p = Parameter::new("enabled", true);
        assert!(p.as_bool().unwrap());
        assert!(matches!(
            p.as_int(),
            Err(BusError::ParameterTypeMismatch { .. })
        ));
        assert_eq!(ParameterValue::from(3_i64).as_int().unwrap(), 3);
        assert_eq!(ParameterValue::from(1.25).as_double().unwrap(), 1.25);
        assert_eq!(ParameterValue::from("x").as_string().unwrap(), "x");
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
        store.undeclare("flag").unwrap();
        assert!(!store.has("flag"));
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
        assert_eq!(
            store.get("max_speed").unwrap().value,
            ParameterValue::Double(1.5)
        );
        assert_eq!(
            store.get("frame_id").unwrap().value,
            ParameterValue::String("base_link".into())
        );
        assert_eq!(
            store.get("enabled").unwrap().value,
            ParameterValue::Bool(true)
        );
        assert_eq!(
            store.get("count").unwrap().value,
            ParameterValue::Integer(3)
        );
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
        assert_eq!(
            store.get("max_speed").unwrap().value,
            ParameterValue::Double(2.5)
        );
        assert_eq!(
            store.get("frame_id").unwrap().value,
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
        assert_eq!(
            store.get("enabled").unwrap().value,
            ParameterValue::Bool(false)
        );
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
        assert_eq!(
            store.get("max_speed").unwrap().value,
            ParameterValue::Double(9.0)
        );
        assert_eq!(
            store.get("frame_id").unwrap().value,
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
        assert_eq!(
            store.get("max_speed").unwrap().value,
            ParameterValue::Double(2.0)
        );

        store.declare("count", ParameterValue::Integer(3)).unwrap();
        store.set("count", ParameterValue::Double(9.0)).unwrap();
        assert_eq!(
            store.get("count").unwrap().value,
            ParameterValue::Integer(9)
        );
        assert!(matches!(
            store.set("count", ParameterValue::Double(1.5)),
            Err(BusError::ParameterTypeMismatch { .. })
        ));
    }
}
