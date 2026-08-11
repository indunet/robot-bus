//! Node construction, discovery, and spin C ABI.

use std::os::raw::{c_char, c_int};
use std::ptr;
use std::time::Duration;

use robot_bus::discovery::{wait as discover_wait, DiscoverOpts as RustDiscoverOpts};
use robot_bus::runtime::{
    Node as RustNode, NodeOptions as RustNodeOptions,
};

use crate::clients::RobotBusShutdownHandle;
use crate::context::{context_ref, RobotBusContext};
use crate::ffi::{
    bus_err, clear_error, cstr_opt, cstr_req, dup_string, err, node_options, ok,
    set_error, robot_bus_free_string,
};

// --- Node -------------------------------------------------------------------

pub(crate) struct RobotBusNode {
    pub(crate) inner: RustNode,
}

#[repr(C)]
pub(crate) struct RobotBusNodeOptions {
    pub host: *const c_char,
    pub transport: *const c_char,
    pub ws_url: *const c_char,
    pub message_xsub: *const c_char,
    pub message_xpub: *const c_char,
    pub service_frontend: *const c_char,
    pub service_backend: *const c_char,
    pub action_backend: *const c_char,
    pub action_frontend: *const c_char,
}

pub(crate) fn parse_node_options(opts: *const RobotBusNodeOptions) -> Result<RustNodeOptions, c_int> {
    if opts.is_null() {
        return Ok(RustNodeOptions::default());
    }
    let o = unsafe { &*opts };
    let host = cstr_opt(o.host).unwrap_or("localhost");
    let transport = cstr_opt(o.transport).unwrap_or("tcp");
    node_options(
        host,
        transport,
        cstr_opt(o.ws_url).map(str::to_string),
        cstr_opt(o.message_xsub).map(str::to_string),
        cstr_opt(o.message_xpub).map(str::to_string),
        cstr_opt(o.service_frontend).map(str::to_string),
        cstr_opt(o.service_backend).map(str::to_string),
        cstr_opt(o.action_backend).map(str::to_string),
        cstr_opt(o.action_frontend).map(str::to_string),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_new(
    name: *const c_char,
    opts: *const RobotBusNodeOptions,
) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let options = match parse_node_options(opts) {
        Ok(o) => o,
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::with_options(name, options),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_new_with_context(
    ctx: *mut RobotBusContext,
    name: *const c_char,
    opts: *const RobotBusNodeOptions,
) -> *mut RobotBusNode {
    clear_error();
    let context = match context_ref(ctx) {
        Ok(c) => c,
        Err(_) => return ptr::null_mut(),
    };
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let options = match parse_node_options(opts) {
        Ok(o) => o,
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::with_context_options(context, name, options),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_tcp(name: *const c_char, host: *const c_char) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let host = cstr_opt(host).unwrap_or("localhost");
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::with_options(name, RustNodeOptions::tcp_at(host)),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_ipc(name: *const c_char, path: *const c_char) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let options = match cstr_opt(path) {
        Some(dir) => RustNodeOptions::ipc_at(dir),
        None => RustNodeOptions::ipc(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::with_options(name, options),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_inproc(
    name: *const c_char,
    prefix: *const c_char,
) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let options = match cstr_opt(prefix) {
        Some(p) => RustNodeOptions::inproc_at(p),
        None => RustNodeOptions::inproc(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::with_options(name, options),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_inproc_with_context(
    ctx: *mut RobotBusContext,
    name: *const c_char,
    prefix: *const c_char,
) -> *mut RobotBusNode {
    clear_error();
    let context = match context_ref(ctx) {
        Ok(c) => c,
        Err(_) => return ptr::null_mut(),
    };
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let node = match cstr_opt(prefix) {
        Some(p) => RustNode::inproc_at_with_context(context, name, p),
        None => RustNode::inproc_with_context(context, name),
    };
    Box::into_raw(Box::new(RobotBusNode { inner: node }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_ws(name: *const c_char) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::ws(name),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_ws_at(
    name: *const c_char,
    url: *const c_char,
) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let url = match cstr_req(url) {
        Ok(u) => u,
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::ws_at(name, url),
    }))
}

#[repr(C)]
pub(crate) struct RobotBusDiscoverOpts {
    pub api_url: *const c_char,
    pub broker_id: *const c_char,
    pub timeout_secs: f64,
}

#[repr(C)]
pub(crate) struct RobotBusAppliedNodeOptions {
    pub host: *mut c_char,
    pub transport: *mut c_char,
    pub ws_url: *mut c_char,
    pub message_xsub: *mut c_char,
    pub message_xpub: *mut c_char,
    pub service_frontend: *mut c_char,
    pub service_backend: *mut c_char,
    pub action_backend: *mut c_char,
    pub action_frontend: *mut c_char,
}

fn parse_discover_opts(opts: *const RobotBusDiscoverOpts) -> Result<RustDiscoverOpts, ()> {
    let mut out = RustDiscoverOpts::default();
    if opts.is_null() {
        return Ok(out);
    }
    let o = unsafe { &*opts };
    if let Some(url) = cstr_opt(o.api_url) {
        if !url.is_empty() {
            out.api_url = url.to_string();
        }
    }
    if let Some(id) = cstr_opt(o.broker_id) {
        if !id.is_empty() {
            out.broker_id = Some(id.to_string());
        }
    }
    if o.timeout_secs > 0.0 {
        out.timeout = Duration::from_secs_f64(o.timeout_secs);
    }
    Ok(out)
}

fn transport_base_options(transport: &str) -> Result<RustNodeOptions, c_int> {
    match transport {
        "tcp" => Ok(RustNodeOptions::tcp()),
        "ipc" => Ok(RustNodeOptions::ipc()),
        "inproc" => Ok(RustNodeOptions::inproc()),
        "ws" => Ok(RustNodeOptions::ws()),
        other => Err(err(format!("unknown transport {other:?}"))),
    }
}

fn apply_discovered(
    transport: &str,
    opts: *const RobotBusDiscoverOpts,
) -> Result<RustNodeOptions, c_int> {
    let discover = match parse_discover_opts(opts) {
        Ok(d) => d,
        Err(()) => return Err(-1),
    };
    let base = transport_base_options(transport)?;
    match discover_wait(discover).and_then(|ann| ann.apply(base)) {
        Ok(o) => Ok(o),
        Err(e) => Err(bus_err(e)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_discover(
    name: *const c_char,
    transport: *const c_char,
    opts: *const RobotBusDiscoverOpts,
) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let transport = match cstr_req(transport) {
        Ok(t) => t,
        Err(_) => return ptr::null_mut(),
    };
    let options = match apply_discovered(transport, opts) {
        Ok(o) => o,
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::with_options(name, options),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_discover_node_options(
    transport: *const c_char,
    opts: *const RobotBusDiscoverOpts,
    out: *mut RobotBusAppliedNodeOptions,
) -> c_int {
    clear_error();
    if out.is_null() {
        return err("null out");
    }
    let transport = match cstr_req(transport) {
        Ok(t) => t,
        Err(_) => return -1,
    };
    let options = match apply_discovered(transport, opts) {
        Ok(o) => o,
        Err(code) => return code,
    };
    unsafe {
        (*out).host = dup_string(&options.host);
        (*out).transport = dup_string(&options.transport);
        (*out).ws_url = options
            .ws_url
            .as_deref()
            .map(dup_string)
            .unwrap_or(ptr::null_mut());
        (*out).message_xsub = options
            .message_xsub
            .as_deref()
            .map(dup_string)
            .unwrap_or(ptr::null_mut());
        (*out).message_xpub = options
            .message_xpub
            .as_deref()
            .map(dup_string)
            .unwrap_or(ptr::null_mut());
        (*out).service_frontend = options
            .service_frontend
            .as_deref()
            .map(dup_string)
            .unwrap_or(ptr::null_mut());
        (*out).service_backend = options
            .service_backend
            .as_deref()
            .map(dup_string)
            .unwrap_or(ptr::null_mut());
        (*out).action_backend = options
            .action_backend
            .as_deref()
            .map(dup_string)
            .unwrap_or(ptr::null_mut());
        (*out).action_frontend = options
            .action_frontend
            .as_deref()
            .map(dup_string)
            .unwrap_or(ptr::null_mut());
    }
    ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_applied_node_options_free(o: *mut RobotBusAppliedNodeOptions) {
    if o.is_null() {
        return;
    }
    unsafe {
        robot_bus_free_string((*o).host);
        robot_bus_free_string((*o).transport);
        robot_bus_free_string((*o).ws_url);
        robot_bus_free_string((*o).message_xsub);
        robot_bus_free_string((*o).message_xpub);
        robot_bus_free_string((*o).service_frontend);
        robot_bus_free_string((*o).service_backend);
        robot_bus_free_string((*o).action_backend);
        robot_bus_free_string((*o).action_frontend);
        (*o).host = ptr::null_mut();
        (*o).transport = ptr::null_mut();
        (*o).ws_url = ptr::null_mut();
        (*o).message_xsub = ptr::null_mut();
        (*o).message_xpub = ptr::null_mut();
        (*o).service_frontend = ptr::null_mut();
        (*o).service_backend = ptr::null_mut();
        (*o).action_backend = ptr::null_mut();
        (*o).action_frontend = ptr::null_mut();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_free(n: *mut RobotBusNode) {
    if !n.is_null() {
        unsafe {
            drop(Box::from_raw(n));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_name(n: *const RobotBusNode) -> *mut c_char {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*n }.inner.name())
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_shutdown_handle(
    n: *mut RobotBusNode,
) -> *mut RobotBusShutdownHandle {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    match unsafe { &mut *n }.inner.shutdown_handle() {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusShutdownHandle { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_shutdown(n: *mut RobotBusNode) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    match unsafe { &mut *n }.inner.shutdown() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_spin_once(n: *mut RobotBusNode, timeout_secs: f64) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(timeout_secs))
    };
    match unsafe { &mut *n }.inner.spin_once(timeout) {
        Ok(true) => {
            clear_error();
            1
        }
        Ok(false) => {
            clear_error();
            0
        }
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_spin(n: *mut RobotBusNode) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    match unsafe { &mut *n }.inner.spin() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_start(n: *mut RobotBusNode) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    match unsafe { &mut *n }.inner.start() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_stop(n: *mut RobotBusNode) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    match unsafe { &mut *n }.inner.stop() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_wait(n: *mut RobotBusNode) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    match unsafe { &mut *n }.inner.wait() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

