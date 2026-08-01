//! Single- and multi-threaded executor C ABI.

use std::os::raw::{c_char, c_int};
use std::ptr;
use std::time::Duration;

use robot_bus::runtime::{
    MultiThreadedExecutor as RustMultiThreadedExecutor,
    SingleThreadedExecutor as RustSingleThreadedExecutor,
};

use crate::clients::RobotBusShutdownHandle;
use crate::context::{context_ref, RobotBusContext};
use crate::ffi::{bus_err, clear_error, cstr_req, err, ok, set_error};
use crate::node::{parse_node_options, RobotBusNode, RobotBusNodeOptions};

pub(crate) struct RobotBusSingleThreadedExecutor {
    pub(crate) inner: RustSingleThreadedExecutor,
}

pub(crate) struct RobotBusMultiThreadedExecutor {
    pub(crate) inner: RustMultiThreadedExecutor,
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_new() -> *mut RobotBusSingleThreadedExecutor {
    clear_error();
    Box::into_raw(Box::new(RobotBusSingleThreadedExecutor {
        inner: RustSingleThreadedExecutor::new(),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_new_with_context(
    ctx: *mut RobotBusContext,
) -> *mut RobotBusSingleThreadedExecutor {
    clear_error();
    let context = match context_ref(ctx) {
        Ok(c) => c.clone(),
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new(RobotBusSingleThreadedExecutor {
        inner: RustSingleThreadedExecutor::with_context(context),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_free(e: *mut RobotBusSingleThreadedExecutor) {
    if !e.is_null() {
        unsafe {
            drop(Box::from_raw(e));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_add_node(
    e: *mut RobotBusSingleThreadedExecutor,
    n: *mut RobotBusNode,
) -> c_int {
    if e.is_null() || n.is_null() {
        return err("null argument");
    }
    match unsafe { &*e }.inner.add_node(&mut unsafe { &mut *n }.inner) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_create_node(
    e: *mut RobotBusSingleThreadedExecutor,
    name: *const c_char,
    opts: *const RobotBusNodeOptions,
) -> *mut RobotBusNode {
    if e.is_null() {
        set_error("null executor");
        return ptr::null_mut();
    }
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let options = match parse_node_options(opts) {
        Ok(o) => o,
        Err(_) => return ptr::null_mut(),
    };
    match unsafe { &*e }.inner.create_node_with_options(name, options) {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusNode { inner }))
        }
        Err(err_) => {
            set_error(err_.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_shutdown_handle(
    e: *mut RobotBusSingleThreadedExecutor,
) -> *mut RobotBusShutdownHandle {
    if e.is_null() {
        set_error("null executor");
        return ptr::null_mut();
    }
    match unsafe { &*e }.inner.shutdown_handle() {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusShutdownHandle { inner }))
        }
        Err(err_) => {
            set_error(err_.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_shutdown(
    e: *mut RobotBusSingleThreadedExecutor,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    match unsafe { &*e }.inner.shutdown() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_spin_once(
    e: *mut RobotBusSingleThreadedExecutor,
    timeout_secs: f64,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(timeout_secs))
    };
    match unsafe { &*e }.inner.spin_once(timeout) {
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
pub extern "C" fn robot_bus_single_threaded_executor_spin(
    e: *mut RobotBusSingleThreadedExecutor,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    match unsafe { &*e }.inner.spin() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_start(
    e: *mut RobotBusSingleThreadedExecutor,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    match unsafe { &*e }.inner.start() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_stop(
    e: *mut RobotBusSingleThreadedExecutor,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    match unsafe { &*e }.inner.stop() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_wait(
    e: *mut RobotBusSingleThreadedExecutor,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    match unsafe { &*e }.inner.wait() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_new(
    num_threads: usize,
) -> *mut RobotBusMultiThreadedExecutor {
    clear_error();
    let n = if num_threads == 0 { 4 } else { num_threads };
    Box::into_raw(Box::new(RobotBusMultiThreadedExecutor {
        inner: RustMultiThreadedExecutor::new(n),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_new_with_context(
    ctx: *mut RobotBusContext,
    num_threads: usize,
) -> *mut RobotBusMultiThreadedExecutor {
    clear_error();
    let context = match context_ref(ctx) {
        Ok(c) => c.clone(),
        Err(_) => return ptr::null_mut(),
    };
    let n = if num_threads == 0 { 4 } else { num_threads };
    Box::into_raw(Box::new(RobotBusMultiThreadedExecutor {
        inner: RustMultiThreadedExecutor::with_context(context, n),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_free(e: *mut RobotBusMultiThreadedExecutor) {
    if !e.is_null() {
        unsafe {
            drop(Box::from_raw(e));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_add_node(
    e: *mut RobotBusMultiThreadedExecutor,
    n: *mut RobotBusNode,
) -> c_int {
    if e.is_null() || n.is_null() {
        return err("null argument");
    }
    match unsafe { &*e }.inner.add_node(&mut unsafe { &mut *n }.inner) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_create_node(
    e: *mut RobotBusMultiThreadedExecutor,
    name: *const c_char,
    opts: *const RobotBusNodeOptions,
) -> *mut RobotBusNode {
    if e.is_null() {
        set_error("null executor");
        return ptr::null_mut();
    }
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    let options = match parse_node_options(opts) {
        Ok(o) => o,
        Err(_) => return ptr::null_mut(),
    };
    match unsafe { &*e }.inner.create_node_with_options(name, options) {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusNode { inner }))
        }
        Err(err_) => {
            set_error(err_.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_shutdown_handle(
    e: *mut RobotBusMultiThreadedExecutor,
) -> *mut RobotBusShutdownHandle {
    if e.is_null() {
        set_error("null executor");
        return ptr::null_mut();
    }
    match unsafe { &*e }.inner.shutdown_handle() {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusShutdownHandle { inner }))
        }
        Err(err_) => {
            set_error(err_.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_shutdown(
    e: *mut RobotBusMultiThreadedExecutor,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    match unsafe { &*e }.inner.shutdown() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_multi_threaded_executor_spin_once(
    e: *mut RobotBusMultiThreadedExecutor,
    timeout_secs: f64,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(timeout_secs))
    };
    match unsafe { &*e }.inner.spin_once(timeout) {
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
pub extern "C" fn robot_bus_multi_threaded_executor_spin(
    e: *mut RobotBusMultiThreadedExecutor,
) -> c_int {
    if e.is_null() {
        return err("null executor");
    }
    match unsafe { &*e }.inner.spin() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

// --- Broker -----------------------------------------------------------------

