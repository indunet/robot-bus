//! Node parameter C ABI.

use std::os::raw::{c_char, c_int};
use std::ptr;

use robot_bus::runtime::{Parameter, ParameterValue};

use crate::ffi::{
    bus_err, clear_error, cstr_req, dup_string, err, ok, robot_bus_free_string,
};
use crate::node::RobotBusNode;

#[repr(C)]
pub(crate) struct RobotBusParameterValue {
    pub type_: c_int,
    pub bool_value: c_int,
    pub integer_value: i64,
    pub double_value: f64,
    pub string_value: *mut c_char,
}

#[repr(C)]
pub(crate) struct RobotBusParameter {
    pub name: *mut c_char,
    pub value: RobotBusParameterValue,
}

const PARAM_BOOL: c_int = 0;
const PARAM_INTEGER: c_int = 1;
const PARAM_DOUBLE: c_int = 2;
const PARAM_STRING: c_int = 3;

fn parameter_value_from_c(v: &RobotBusParameterValue) -> Result<ParameterValue, c_int> {
    match v.type_ {
        PARAM_BOOL => Ok(ParameterValue::Bool(v.bool_value != 0)),
        PARAM_INTEGER => Ok(ParameterValue::Integer(v.integer_value)),
        PARAM_DOUBLE => Ok(ParameterValue::Double(v.double_value)),
        PARAM_STRING => {
            let s = cstr_req(v.string_value)?;
            Ok(ParameterValue::String(s.to_string()))
        }
        _ => Err(err("invalid parameter type")),
    }
}

fn parameter_value_to_c(value: ParameterValue) -> RobotBusParameterValue {
    match value {
        ParameterValue::Bool(b) => RobotBusParameterValue {
            type_: PARAM_BOOL,
            bool_value: if b { 1 } else { 0 },
            integer_value: 0,
            double_value: 0.0,
            string_value: ptr::null_mut(),
        },
        ParameterValue::Integer(i) => RobotBusParameterValue {
            type_: PARAM_INTEGER,
            bool_value: 0,
            integer_value: i,
            double_value: 0.0,
            string_value: ptr::null_mut(),
        },
        ParameterValue::Double(d) => RobotBusParameterValue {
            type_: PARAM_DOUBLE,
            bool_value: 0,
            integer_value: 0,
            double_value: d,
            string_value: ptr::null_mut(),
        },
        ParameterValue::String(s) => RobotBusParameterValue {
            type_: PARAM_STRING,
            bool_value: 0,
            integer_value: 0,
            double_value: 0.0,
            string_value: dup_string(&s),
        },
    }
}

fn free_parameter_value(v: &mut RobotBusParameterValue) {
    if !v.string_value.is_null() {
        robot_bus_free_string(v.string_value);
        v.string_value = ptr::null_mut();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_declare_parameter(
    n: *mut RobotBusNode,
    name: *const c_char,
    value: *const RobotBusParameterValue,
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    if value.is_null() {
        return err("null parameter value");
    }
    let name = match cstr_req(name) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let pv = match parameter_value_from_c(unsafe { &*value }) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match unsafe { &mut *n }.inner.declare_parameter(name, pv) {
        Ok(_) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_set_parameter(
    n: *mut RobotBusNode,
    name: *const c_char,
    value: *const RobotBusParameterValue,
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    if value.is_null() {
        return err("null parameter value");
    }
    let name = match cstr_req(name) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let pv = match parameter_value_from_c(unsafe { &*value }) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match unsafe { &mut *n }
        .inner
        .set_parameter(Parameter::new(name, pv))
    {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_get_parameter(
    n: *mut RobotBusNode,
    name: *const c_char,
    out: *mut RobotBusParameterValue,
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    if out.is_null() {
        return err("null out");
    }
    let name = match cstr_req(name) {
        Ok(s) => s,
        Err(e) => return e,
    };
    match unsafe { &*n }.inner.get_parameter(name) {
        Ok(p) => {
            unsafe { *out = parameter_value_to_c(p.value) };
            ok()
        }
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_has_parameter(
    n: *const RobotBusNode,
    name: *const c_char,
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    let name = match cstr_req(name) {
        Ok(s) => s,
        Err(e) => return e,
    };
    clear_error();
    if unsafe { &*n }.inner.has_parameter(name) {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_list_parameters(
    n: *mut RobotBusNode,
    out: *mut *mut RobotBusParameter,
    out_count: *mut usize,
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    if out.is_null() || out_count.is_null() {
        return err("null out");
    }
    let list = unsafe { &*n }.inner.list_all_parameters();
    let count = list.len();
    if count == 0 {
        unsafe {
            *out = ptr::null_mut();
            *out_count = 0;
        }
        return ok();
    }
    let mut buf = Vec::with_capacity(count);
    for p in list {
        buf.push(RobotBusParameter {
            name: dup_string(&p.name),
            value: parameter_value_to_c(p.value),
        });
    }
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    unsafe {
        *out = ptr;
        *out_count = count;
    }
    ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_parameters_free(params: *mut RobotBusParameter, count: usize) {
    if params.is_null() || count == 0 {
        return;
    }
    let mut vec = unsafe { Vec::from_raw_parts(params, count, count) };
    for p in &mut vec {
        if !p.name.is_null() {
            robot_bus_free_string(p.name);
            p.name = ptr::null_mut();
        }
        free_parameter_value(&mut p.value);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_load_parameters_from_yaml(
    n: *mut RobotBusNode,
    path: *const c_char,
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    let path = match cstr_req(path) {
        Ok(s) => s,
        Err(e) => return e,
    };
    match unsafe { &mut *n }.inner.load_parameters_from_yaml_file(path) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_load_parameters_from_yaml_str(
    n: *mut RobotBusNode,
    yaml: *const c_char,
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    let yaml = match cstr_req(yaml) {
        Ok(s) => s,
        Err(e) => return e,
    };
    match unsafe { &mut *n }.inner.load_parameters_from_yaml_str(yaml) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

