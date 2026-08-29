//! Node entity creation (pub/sub/timer/service/action) C ABI.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use robot_bus::runtime::{
    ActionGoalHandler, CallbackGroupType, QosProfile, ServiceHandler, TimerCallback,
};

use crate::clients::{
    RobotBusActionClient, RobotBusActionHandler, RobotBusActionPhase, RobotBusActionServerHandle,
    RobotBusCallbackGroup, RobotBusMsgCallback, RobotBusServiceClient, RobotBusServiceHandle,
    RobotBusServiceHandler, RobotBusSubscriptionHandle, RobotBusTimerCallback, RobotBusTimerHandle,
    RobotBusTopicPublisher,
};
use crate::ffi::{
    bus_err, clear_error, cstr_req, err, ok, set_error, robot_bus_free_string,
};
use crate::node::RobotBusNode;

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_callback_group(
    n: *mut RobotBusNode,
    kind: c_int,
) -> *mut RobotBusCallbackGroup {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    clear_error();
    let k = if kind == 1 {
        CallbackGroupType::Reentrant
    } else {
        CallbackGroupType::MutuallyExclusive
    };
    Box::into_raw(Box::new(RobotBusCallbackGroup {
        inner: unsafe { &*n }.inner.create_callback_group(k),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_publisher(
    n: *mut RobotBusNode,
    topic: *const c_char,
) -> *mut RobotBusTopicPublisher {
    robot_bus_node_create_publisher_with_qos(n, topic, 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_publisher_with_qos(
    n: *mut RobotBusNode,
    topic: *const c_char,
    depth: i32,
) -> *mut RobotBusTopicPublisher {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let topic = match cstr_req(topic) {
        Ok(t) => t,
        Err(_) => return ptr::null_mut(),
    };
    let result = if depth > 0 {
        unsafe { &mut *n }
            .inner
            .create_publisher_raw_with_qos(topic, QosProfile::keep_last(depth))
    } else {
        unsafe { &mut *n }.inner.create_publisher_raw(topic)
    };
    match result {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusTopicPublisher { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_subscription(
    n: *mut RobotBusNode,
    topic: *const c_char,
    callback: RobotBusMsgCallback,
    user: *mut c_void,
    group: *const RobotBusCallbackGroup,
) -> *mut RobotBusSubscriptionHandle {
    robot_bus_node_create_subscription_with_qos(n, topic, callback, user, group, 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_subscription_with_qos(
    n: *mut RobotBusNode,
    topic: *const c_char,
    callback: RobotBusMsgCallback,
    user: *mut c_void,
    group: *const RobotBusCallbackGroup,
    depth: i32,
) -> *mut RobotBusSubscriptionHandle {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let Some(cb_fn) = callback else {
        set_error("null callback");
        return ptr::null_mut();
    };
    let topic = match cstr_req(topic) {
        Ok(t) => t,
        Err(_) => return ptr::null_mut(),
    };
    // user pointer is assumed to outlive the subscription (caller responsibility).
    let user = user as usize;
    let cb: robot_bus::runtime::MessageCallback = Arc::new(move |payload| {
        unsafe {
            cb_fn(
                payload.as_ptr(),
                payload.len(),
                user as *mut c_void,
            );
        }
    });
    let group = if group.is_null() {
        None
    } else {
        Some(&unsafe { &*group }.inner)
    };
    let result = if depth > 0 {
        unsafe { &mut *n }.inner.create_subscription_raw_with_qos(
            topic,
            QosProfile::keep_last(depth),
            cb,
            group,
        )
    } else {
        unsafe { &mut *n }
            .inner
            .create_subscription_raw(topic, cb, group)
    };
    match result {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusSubscriptionHandle { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_destroy_subscription(
    n: *mut RobotBusNode,
    handle: *mut RobotBusSubscriptionHandle,
) -> c_int {
    if n.is_null() || handle.is_null() {
        return err("null argument");
    }
    let inner = unsafe { (*handle).inner };
    match unsafe { &mut *n }.inner.destroy_subscription(inner) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_timer(
    n: *mut RobotBusNode,
    period_secs: f64,
    callback: RobotBusTimerCallback,
    user: *mut c_void,
    group: *const RobotBusCallbackGroup,
) -> *mut RobotBusTimerHandle {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let Some(cb_fn) = callback else {
        set_error("null callback");
        return ptr::null_mut();
    };
    let user = user as usize;
    let cb: TimerCallback = Arc::new(move || unsafe {
        cb_fn(user as *mut c_void);
    });
    let group = if group.is_null() {
        None
    } else {
        Some(&unsafe { &*group }.inner)
    };
    match unsafe { &mut *n }.inner.create_timer(
        Duration::from_secs_f64(period_secs),
        cb,
        group,
    ) {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusTimerHandle { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_cancel_timer(
    n: *mut RobotBusNode,
    handle: *const RobotBusTimerHandle,
) -> c_int {
    if n.is_null() || handle.is_null() {
        return err("null argument");
    }
    match unsafe { &mut *n }
        .inner
        .cancel_timer(unsafe { &*handle }.inner)
    {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_service(
    n: *mut RobotBusNode,
    service_name: *const c_char,
    handler: RobotBusServiceHandler,
    user: *mut c_void,
    group: *const RobotBusCallbackGroup,
) -> *mut RobotBusServiceHandle {
    robot_bus_node_create_service_with_qos(n, service_name, handler, user, group, 0)
}

/** `depth <= 0` keeps the node RPC HWM; `depth > 0` maps to KeepLast. */
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_service_with_qos(
    n: *mut RobotBusNode,
    service_name: *const c_char,
    handler: RobotBusServiceHandler,
    user: *mut c_void,
    group: *const RobotBusCallbackGroup,
    depth: i32,
) -> *mut RobotBusServiceHandle {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let Some(handler_fn) = handler else {
        set_error("null handler");
        return ptr::null_mut();
    };
    let service_name = match cstr_req(service_name) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let user = user as usize;
    let cb: ServiceHandler = Arc::new(move |body| {
        let mut out_data: *mut u8 = ptr::null_mut();
        let mut out_len: usize = 0;
        let rc = unsafe {
            handler_fn(
                body.as_ptr(),
                body.len(),
                &mut out_data,
                &mut out_len,
                user as *mut c_void,
            )
        };
        if rc != 0 || out_data.is_null() {
            return Vec::new();
        }
        unsafe { Vec::from_raw_parts(out_data, out_len, out_len) }
    });
    let group = if group.is_null() {
        None
    } else {
        Some(&unsafe { &*group }.inner)
    };
    match if depth > 0 {
        unsafe { &mut *n }.inner.create_service_raw_with_qos(
            service_name,
            QosProfile::keep_last(depth),
            cb,
            group,
        )
    } else {
        unsafe { &mut *n }
            .inner
            .create_service_raw(service_name, cb, group)
    } {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusServiceHandle { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_destroy_service(
    n: *mut RobotBusNode,
    handle: *mut RobotBusServiceHandle,
) -> c_int {
    if n.is_null() || handle.is_null() {
        return err("null argument");
    }
    match unsafe { &mut *n }
        .inner
        .destroy_service(&unsafe { &*handle }.inner)
    {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_client(
    n: *mut RobotBusNode,
    service_name: *const c_char,
) -> *mut RobotBusServiceClient {
    robot_bus_node_create_client_with_qos(n, service_name, 0)
}

/** `depth <= 0` keeps the node RPC HWM; `depth > 0` maps to KeepLast. */
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_client_with_qos(
    n: *mut RobotBusNode,
    service_name: *const c_char,
    depth: i32,
) -> *mut RobotBusServiceClient {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let service_name = match cstr_req(service_name) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match if depth > 0 {
        unsafe { &mut *n }
            .inner
            .create_client_raw_with_qos(service_name, QosProfile::keep_last(depth))
    } else {
        unsafe { &mut *n }.inner.create_client_raw(service_name)
    } {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusServiceClient { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_action_server(
    n: *mut RobotBusNode,
    action_name: *const c_char,
    handler: RobotBusActionHandler,
    user: *mut c_void,
    group: *const RobotBusCallbackGroup,
) -> *mut RobotBusActionServerHandle {
    robot_bus_node_create_action_server_with_qos(n, action_name, handler, user, group, 0)
}

/** `depth <= 0` keeps the node action HWM; `depth > 0` maps to KeepLast. */
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_action_server_with_qos(
    n: *mut RobotBusNode,
    action_name: *const c_char,
    handler: RobotBusActionHandler,
    user: *mut c_void,
    group: *const RobotBusCallbackGroup,
    depth: i32,
) -> *mut RobotBusActionServerHandle {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let Some(handler_fn) = handler else {
        set_error("null handler");
        return ptr::null_mut();
    };
    let action_name = match cstr_req(action_name) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let user = user as usize;
    let cb: ActionGoalHandler = Arc::new(move |payload| {
        let mut out_phases: *mut RobotBusActionPhase = ptr::null_mut();
        let mut out_count: usize = 0;
        let rc = unsafe {
            handler_fn(
                payload.as_ptr(),
                payload.len(),
                &mut out_phases,
                &mut out_count,
                user as *mut c_void,
            )
        };
        if rc != 0 || out_phases.is_null() || out_count == 0 {
            return Vec::new();
        }
        let phases = unsafe { Vec::from_raw_parts(out_phases, out_count, out_count) };
        phases
            .into_iter()
            .map(|p| {
                let phase = if p.phase.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(p.phase) }
                        .to_string_lossy()
                        .into_owned()
                };
                let body = if p.body.is_null() || p.body_len == 0 {
                    Vec::new()
                } else {
                    unsafe { Vec::from_raw_parts(p.body, p.body_len, p.body_len) }
                };
                robot_bus_free_string(p.phase);
                (phase, body)
            })
            .collect()
    });
    let group = if group.is_null() {
        None
    } else {
        Some(&unsafe { &*group }.inner)
    };
    match if depth > 0 {
        unsafe { &mut *n }.inner.create_action_server_raw_with_qos(
            action_name,
            QosProfile::keep_last(depth),
            cb,
            group,
        )
    } else {
        unsafe { &mut *n }
            .inner
            .create_action_server_raw(action_name, cb, group)
    } {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusActionServerHandle { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_destroy_action_server(
    n: *mut RobotBusNode,
    handle: *mut RobotBusActionServerHandle,
) -> c_int {
    if n.is_null() || handle.is_null() {
        return err("null argument");
    }
    match unsafe { &mut *n }
        .inner
        .destroy_action_server(&unsafe { &*handle }.inner)
    {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_action_client(
    n: *mut RobotBusNode,
    action_name: *const c_char,
) -> *mut RobotBusActionClient {
    robot_bus_node_create_action_client_with_qos(n, action_name, 0)
}

/** `depth <= 0` keeps the node action HWM; `depth > 0` maps to KeepLast. */
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_action_client_with_qos(
    n: *mut RobotBusNode,
    action_name: *const c_char,
    depth: i32,
) -> *mut RobotBusActionClient {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let action_name = match cstr_req(action_name) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match if depth > 0 {
        unsafe { &mut *n }
            .inner
            .create_action_client_raw_with_qos(action_name, QosProfile::keep_last(depth))
    } else {
        unsafe { &mut *n }.inner.create_action_client_raw(action_name)
    } {
        Ok(inner) => {
            clear_error();
            Box::into_raw(Box::new(RobotBusActionClient { inner }))
        }
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_connect_action_client(n: *mut RobotBusNode) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    match unsafe { &mut *n }.inner.connect_action_client() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}
