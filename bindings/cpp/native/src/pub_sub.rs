//! Low-level message-bus publisher / subscriber C ABI.

use std::os::raw::{c_char, c_int};
use std::ptr;

use robot_bus::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};

use crate::ffi::{bytes_slice, bus_err, clear_error, cstr_opt, cstr_req, dup_bytes, dup_string, err, ok, set_error};

use std::time::Duration;

// --- Publisher / Subscriber -------------------------------------------------

pub(crate) struct RobotBusPublisher {
    pub(crate) inner: RustPublisher,
}

pub(crate) struct RobotBusSubscriber {
    pub(crate) inner: RustSubscriber,
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_publisher_new(
    endpoint: *const c_char,
) -> *mut RobotBusPublisher {
    clear_error();
    let ep = cstr_opt(endpoint);
    match RustPublisher::new(ep) {
        Ok(inner) => Box::into_raw(Box::new(RobotBusPublisher { inner })),
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_publisher_free(p: *mut RobotBusPublisher) {
    if !p.is_null() {
        unsafe {
            drop(Box::from_raw(p));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_publisher_publish(
    p: *mut RobotBusPublisher,
    topic: *const c_char,
    data: *const u8,
    len: usize,
) -> c_int {
    if p.is_null() {
        return err("null publisher");
    }
    let topic = match cstr_req(topic) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let bytes = match bytes_slice(data, len) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let pub_ = unsafe { &*p };
    match pub_.inner.publish(topic, bytes) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_publisher_endpoint(p: *const RobotBusPublisher) -> *mut c_char {
    if p.is_null() {
        set_error("null publisher");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*p }.inner.endpoint())
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_subscriber_new(
    endpoint: *const c_char,
) -> *mut RobotBusSubscriber {
    clear_error();
    let ep = cstr_opt(endpoint);
    match RustSubscriber::new(ep) {
        Ok(inner) => Box::into_raw(Box::new(RobotBusSubscriber { inner })),
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_subscriber_free(s: *mut RobotBusSubscriber) {
    if !s.is_null() {
        unsafe {
            drop(Box::from_raw(s));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_subscriber_subscribe(
    s: *mut RobotBusSubscriber,
    topic: *const c_char,
) -> c_int {
    if s.is_null() {
        return err("null subscriber");
    }
    let topic = match cstr_req(topic) {
        Ok(t) => t,
        Err(e) => return e,
    };
    match unsafe { &*s }.inner.subscribe(topic) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_subscriber_unsubscribe(
    s: *mut RobotBusSubscriber,
    topic: *const c_char,
) -> c_int {
    if s.is_null() {
        return err("null subscriber");
    }
    let topic = match cstr_req(topic) {
        Ok(t) => t,
        Err(e) => return e,
    };
    match unsafe { &*s }.inner.unsubscribe(topic) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_subscriber_receive(
    s: *mut RobotBusSubscriber,
    timeout_secs: f64,
    out_topic: *mut *mut c_char,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if s.is_null() || out_topic.is_null() || out_data.is_null() || out_len.is_null() {
        return err("null argument");
    }
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(timeout_secs))
    };
    match unsafe { &*s }.inner.receive(timeout) {
        Ok((topic, payload)) => {
            unsafe {
                *out_topic = dup_string(&topic);
                let (ptr, len) = dup_bytes(&payload);
                *out_data = ptr;
                *out_len = len;
            }
            ok()
        }
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_subscriber_endpoint(s: *const RobotBusSubscriber) -> *mut c_char {
    if s.is_null() {
        set_error("null subscriber");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*s }.inner.endpoint())
}

