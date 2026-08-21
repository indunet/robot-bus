//! Shared helpers for napi bindings.

use napi::bindgen_prelude::*;
use robot_bus::action_bus::ActionKind;
use robot_bus::errors::BusError;
use robot_bus::runtime::{NodeOptions as RustNodeOptions, Parameter, ParameterValue};

pub(crate) fn bus_err(err: BusError) -> Error {
    Error::from_reason(err.to_string())
}

pub(crate) fn anyhow_err(err: anyhow::Error) -> Error {
    Error::from_reason(err.to_string())
}

pub(crate) fn map_endpoint_err(err: String) -> Error {
    Error::from_reason(err)
}

pub(crate) fn parameter_value_from_js(value: Unknown) -> Result<ParameterValue> {
    match value.get_type()? {
        ValueType::Boolean => Ok(ParameterValue::Bool(value.coerce_to_bool()?.get_value()?)),
        ValueType::Number => {
            let n = value.coerce_to_number()?.get_double()?;
            if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                Ok(ParameterValue::Integer(n as i64))
            } else {
                Ok(ParameterValue::Double(n))
            }
        }
        ValueType::String => Ok(ParameterValue::String(
            value.coerce_to_string()?.into_utf8()?.into_owned()?,
        )),
        other => Err(Error::from_reason(format!(
            "parameter value must be bool, number, or string; got {other:?}"
        ))),
    }
}

pub(crate) fn parameter_value_to_js(env: &Env, value: ParameterValue) -> Result<Unknown> {
    Ok(match value {
        ParameterValue::Bool(v) => env.get_boolean(v)?.into_unknown(),
        ParameterValue::Integer(v) => env.create_int64(v)?.into_unknown(),
        ParameterValue::Double(v) => env.create_double(v)?.into_unknown(),
        ParameterValue::String(v) => env.create_string(&v)?.into_unknown(),
    })
}

pub(crate) fn parameter_to_js(env: &Env, param: Parameter) -> Result<Unknown> {
    let mut obj = env.create_object()?;
    obj.set_named_property("name", env.create_string(&param.name)?)?;
    obj.set_named_property("value", parameter_value_to_js(env, param.value)?)?;
    Ok(obj.into_unknown())
}

pub(crate) fn node_options(
    host: &str,
    transport: &str,
    ws_url: Option<String>,
    message_xsub: Option<String>,
    message_xpub: Option<String>,
    service_frontend: Option<String>,
    service_backend: Option<String>,
    action_backend: Option<String>,
    action_frontend: Option<String>,
) -> Result<RustNodeOptions> {
    if transport == "ws" {
        return Ok(match ws_url {
            Some(url) => RustNodeOptions::ws_at(url),
            None => RustNodeOptions::ws(),
        });
    }
    if ws_url.is_some() {
        return Err(Error::from_reason(
            "ws_url is only valid when transport=\"ws\"",
        ));
    }
    Ok(RustNodeOptions {
        host: host.into(),
        transport: transport.into(),
        ws_url: None,
        console_url: None,
        message_xsub,
        message_xpub,
        service_frontend,
        service_backend,
        action_backend,
        action_frontend,
    })
}

pub(crate) fn normalize_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
    }
}

pub(crate) fn action_kind_str(kind: ActionKind) -> &'static str {
    match kind {
        ActionKind::Goal => "GOAL",
        ActionKind::Feedback => "FEEDBACK",
        ActionKind::Result => "RESULT",
        ActionKind::Cancel => "CANCEL",
    }
}
