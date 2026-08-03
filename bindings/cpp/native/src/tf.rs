//! TF buffer / listener C ABI (protobuf bytes at the boundary).

use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::{Arc, Mutex};

use prost::Message;
use robot_bus::geometry_msgs::msg::v1::TransformStamped;
use robot_bus::tf::{Buffer, SharedBuffer, TfListener};
use robot_bus::tf2_msgs::msg::v1::TfMessage;

use crate::ffi::{
    bytes_slice, clear_error, cstr_req, dup_bytes, dup_string, err, ok, set_error,
};
use crate::node::RobotBusNode;

pub(crate) struct RobotBusTfBuffer {
    buffer: SharedBuffer,
}

pub(crate) struct RobotBusTfListener {
    inner: TfListener,
}

fn lock_buffer(buf: &SharedBuffer) -> Result<std::sync::MutexGuard<'_, Buffer>, c_int> {
    buf.lock().map_err(|_| err("tf buffer lock poisoned"))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_buffer_new() -> *mut RobotBusTfBuffer {
    clear_error();
    Box::into_raw(Box::new(RobotBusTfBuffer {
        buffer: Arc::new(Mutex::new(Buffer::new())),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_buffer_free(buf: *mut RobotBusTfBuffer) {
    if !buf.is_null() {
        unsafe {
            drop(Box::from_raw(buf));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_buffer_clear(buf: *mut RobotBusTfBuffer) -> c_int {
    if buf.is_null() {
        return err("null tf buffer");
    }
    let buf = unsafe { &*buf };
    match lock_buffer(&buf.buffer) {
        Ok(mut guard) => {
            guard.clear();
            ok()
        }
        Err(e) => e,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_buffer_set_transform_msg(
    buf: *mut RobotBusTfBuffer,
    data: *const u8,
    len: usize,
    is_static: c_int,
) -> c_int {
    if buf.is_null() {
        return err("null tf buffer");
    }
    let bytes = match bytes_slice(data, len) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let msg = match TfMessage::decode(bytes) {
        Ok(m) => m,
        Err(e) => return err(format!("decode TFMessage: {e}")),
    };
    let buf = unsafe { &*buf };
    match lock_buffer(&buf.buffer) {
        Ok(mut guard) => {
            guard.set_transform_msg(&msg, is_static != 0);
            ok()
        }
        Err(e) => e,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_buffer_lookup_transform(
    buf: *mut RobotBusTfBuffer,
    target: *const c_char,
    source: *const c_char,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if buf.is_null() || out_data.is_null() || out_len.is_null() {
        return err("null argument");
    }
    let target = match cstr_req(target) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let source = match cstr_req(source) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let buf = unsafe { &*buf };
    let stamped: TransformStamped = match lock_buffer(&buf.buffer) {
        Ok(guard) => match guard.lookup_transform(target, source, None) {
            Ok(t) => t,
            Err(e) => return err(e.to_string()),
        },
        Err(e) => return e,
    };
    let encoded = stamped.encode_to_vec();
    let (ptr, len) = dup_bytes(&encoded);
    unsafe {
        *out_data = ptr;
        *out_len = len;
    }
    ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_buffer_can_transform(
    buf: *mut RobotBusTfBuffer,
    target: *const c_char,
    source: *const c_char,
) -> c_int {
    if buf.is_null() {
        set_error("null tf buffer");
        return 0;
    }
    let target = match cstr_req(target) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let source = match cstr_req(source) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let buf = unsafe { &*buf };
    match lock_buffer(&buf.buffer) {
        Ok(guard) => {
            clear_error();
            if guard.can_transform(target, source) {
                1
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

/// Newline-separated sorted frame ids. Caller frees with `robot_bus_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_buffer_frames(buf: *mut RobotBusTfBuffer) -> *mut c_char {
    if buf.is_null() {
        set_error("null tf buffer");
        return ptr::null_mut();
    }
    let buf = unsafe { &*buf };
    match lock_buffer(&buf.buffer) {
        Ok(guard) => {
            clear_error();
            let joined = guard.frames().join("\n");
            dup_string(&joined)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_listener_new(
    node: *mut RobotBusNode,
    tf_topic: *const c_char,
    tf_static_topic: *const c_char,
) -> *mut RobotBusTfListener {
    clear_error();
    if node.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let tf_topic = match cstr_req(tf_topic) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let tf_static_topic = match cstr_req(tf_static_topic) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let node = unsafe { &mut *node };
    match TfListener::new(&mut node.inner, tf_topic, tf_static_topic) {
        Ok(inner) => Box::into_raw(Box::new(RobotBusTfListener { inner })),
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_listener_with_defaults(
    node: *mut RobotBusNode,
) -> *mut RobotBusTfListener {
    clear_error();
    if node.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let node = unsafe { &mut *node };
    match TfListener::with_defaults(&mut node.inner) {
        Ok(inner) => Box::into_raw(Box::new(RobotBusTfListener { inner })),
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_listener_free(listener: *mut RobotBusTfListener) {
    if !listener.is_null() {
        unsafe {
            drop(Box::from_raw(listener));
        }
    }
}

/// Shared buffer handle (`Arc` clone). Caller must free with `robot_bus_tf_buffer_free`.
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_tf_listener_buffer(
    listener: *mut RobotBusTfListener,
) -> *mut RobotBusTfBuffer {
    clear_error();
    if listener.is_null() {
        set_error("null tf listener");
        return ptr::null_mut();
    }
    let listener = unsafe { &*listener };
    Box::into_raw(Box::new(RobotBusTfBuffer {
        buffer: listener.inner.buffer(),
    }))
}
