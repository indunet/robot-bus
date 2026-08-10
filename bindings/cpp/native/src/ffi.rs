//! Shared C ABI helpers (errors, strings, endpoints).

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::slice;

use robot_bus::errors::BusError;
use robot_bus::runtime::NodeOptions as RustNodeOptions;
use robot_bus::transports;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

pub(crate) fn set_error(msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    let c = CString::new(msg.replace('\0', "")).unwrap_or_else(|_| CString::new("error").unwrap());
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(c));
}

pub(crate) fn last_error_message() -> Option<String> {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|s| s.to_string_lossy().into_owned())
    })
}

pub(crate) fn clear_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
}

pub(crate) fn ok() -> c_int {
    clear_error();
    0
}

pub(crate) fn err(msg: impl AsRef<str>) -> c_int {
    set_error(msg);
    -1
}

pub(crate) fn bus_err(e: BusError) -> c_int {
    err(e.to_string())
}

pub(crate) fn anyhow_err(e: anyhow::Error) -> c_int {
    err(e.to_string())
}

pub(crate) fn cstr_opt<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(p) }.to_str().ok()
    }
}

pub(crate) fn cstr_req<'a>(p: *const c_char) -> Result<&'a str, c_int> {
    cstr_opt(p).ok_or_else(|| err("null string"))
}

pub(crate) fn bytes_slice<'a>(data: *const u8, len: usize) -> Result<&'a [u8], c_int> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(err("null bytes"));
    }
    Ok(unsafe { slice::from_raw_parts(data, len) })
}

pub(crate) fn normalize_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
    }
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
) -> Result<RustNodeOptions, c_int> {
    if transport == "ws" {
        return Ok(match ws_url {
            Some(url) => RustNodeOptions::ws_at(url),
            None => RustNodeOptions::ws(),
        });
    }
    if ws_url.is_some() {
        return Err(err("ws_url is only valid when transport=\"grpc\""));
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

// --- exported helpers -------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| match slot.borrow().as_ref() {
        Some(s) => s.as_ptr(),
        None => ptr::null(),
    })
}

/// Set the thread-local last error string (for C++ mapper callbacks).
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_set_error(msg: *const c_char) {
    match cstr_opt(msg) {
        Some(m) => set_error(m),
        None => set_error("error"),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_free_bytes(data: *mut u8, len: usize) {
    if !data.is_null() && len > 0 {
        unsafe {
            drop(Vec::from_raw_parts(data, len, len));
        }
    }
}

/// Allocate `len` bytes with Rust's global allocator (pair with [`robot_bus_free_bytes`]
/// or hand ownership to service/action reply callbacks that use `Vec::from_raw_parts`).
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_alloc_bytes(len: usize) -> *mut u8 {
    if len == 0 {
        return ptr::null_mut();
    }
    let mut v = vec![0u8; len];
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    ptr
}

/// Duplicate a C string for FFI ownership (pair with [`robot_bus_free_string`]).
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_dup_string(s: *const c_char) -> *mut c_char {
    match cstr_opt(s) {
        Some(v) => dup_string(v),
        None => dup_string(""),
    }
}

pub(crate) fn dup_string(s: &str) -> *mut c_char {
    CString::new(s.replace('\0', ""))
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

pub(crate) fn dup_bytes(b: &[u8]) -> (*mut u8, usize) {
    let mut v = b.to_vec();
    let len = v.len();
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    (ptr, len)
}

// --- endpoints --------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_message_xsub_endpoint(
    host: *const c_char,
    transport: *const c_char,
    out: *mut *mut c_char,
) -> c_int {
    if out.is_null() {
        return err("null out");
    }
    let host = cstr_opt(host).unwrap_or("localhost");
    let transport = cstr_opt(transport).unwrap_or("tcp");
    match transports::message_xsub_endpoint(host, transport) {
        Ok(s) => {
            unsafe { *out = dup_string(&s) };
            ok()
        }
        Err(e) => err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_message_xpub_endpoint(
    host: *const c_char,
    transport: *const c_char,
    out: *mut *mut c_char,
) -> c_int {
    if out.is_null() {
        return err("null out");
    }
    let host = cstr_opt(host).unwrap_or("localhost");
    let transport = cstr_opt(transport).unwrap_or("tcp");
    match transports::message_xpub_endpoint(host, transport) {
        Ok(s) => {
            unsafe { *out = dup_string(&s) };
            ok()
        }
        Err(e) => err(e),
    }
}

