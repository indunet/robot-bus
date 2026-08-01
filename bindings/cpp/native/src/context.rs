//! Shared ZMQ context C ABI.

use std::os::raw::c_int;
use std::ptr;

use robot_bus::runtime::Context as RustContext;

use crate::ffi::{clear_error, err, set_error};

// --- Context ----------------------------------------------------------------

pub(crate) struct RobotBusContext {
    pub(crate) inner: RustContext,
}

pub(crate) fn context_ref(ctx: *mut RobotBusContext) -> Result<&'static RustContext, c_int> {
    if ctx.is_null() {
        Err(err("null context"))
    } else {
        Ok(&unsafe { &*ctx }.inner)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_context_new() -> *mut RobotBusContext {
    clear_error();
    Box::into_raw(Box::new(RobotBusContext {
        inner: RustContext::new(),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_context_free(c: *mut RobotBusContext) {
    if !c.is_null() {
        unsafe {
            drop(Box::from_raw(c));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_context_clone(c: *const RobotBusContext) -> *mut RobotBusContext {
    if c.is_null() {
        set_error("null context");
        return ptr::null_mut();
    }
    clear_error();
    Box::into_raw(Box::new(RobotBusContext {
        inner: unsafe { &*c }.inner.clone(),
    }))
}

