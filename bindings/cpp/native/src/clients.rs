//! Topic publisher, service/action clients, and related handle C ABI.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use robot_bus::action_bus::ActionKind;
use robot_bus::runtime::{
    CallbackGroup, CallbackGroupType, NodeActionClientRaw as RustNodeActionClient,
    NodeServiceClientRaw as RustNodeServiceClient, RawActionFeedbackCallback,
    RawGoalHandle as RustActionGoalHandle, ShutdownHandle as RustShutdownHandle,
    TimerHandle as RustTimerHandle, TopicPublisherRaw as RustTopicPublisher,
};

use crate::ffi::{
    bus_err, bytes_slice, clear_error, cstr_opt, dup_bytes, dup_string, err, ok,
    robot_bus_free_bytes, robot_bus_free_string, set_error,
};

// --- Shutdown / Timer / CallbackGroup ---------------------------------------

pub(crate) struct RobotBusShutdownHandle {
    pub(crate) inner: RustShutdownHandle,
}

pub(crate) struct RobotBusTimerHandle {
    pub(crate) inner: RustTimerHandle,
}

pub(crate) struct RobotBusCallbackGroup {
    pub(crate) inner: CallbackGroup,
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_shutdown_handle_free(h: *mut RobotBusShutdownHandle) {
    if !h.is_null() {
        unsafe {
            drop(Box::from_raw(h));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_shutdown_handle_shutdown(h: *mut RobotBusShutdownHandle) {
    if !h.is_null() {
        unsafe { &*h }.inner.shutdown();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_shutdown_handle_is_running(h: *const RobotBusShutdownHandle) -> c_int {
    if h.is_null() {
        return 0;
    }
    if unsafe { &*h }.inner.is_running() {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_timer_handle_free(h: *mut RobotBusTimerHandle) {
    if !h.is_null() {
        unsafe {
            drop(Box::from_raw(h));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_callback_group_free(g: *mut RobotBusCallbackGroup) {
    if !g.is_null() {
        unsafe {
            drop(Box::from_raw(g));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_callback_group_id(g: *const RobotBusCallbackGroup) -> u64 {
    if g.is_null() {
        return 0;
    }
    unsafe { &*g }.inner.id()
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_callback_group_kind(g: *const RobotBusCallbackGroup) -> c_int {
    if g.is_null() {
        return 0;
    }
    match unsafe { &*g }.inner.kind() {
        CallbackGroupType::MutuallyExclusive => 0,
        CallbackGroupType::Reentrant => 1,
    }
}

// --- TopicPublisher / ServiceClient / ActionClient --------------------------

pub(crate) struct RobotBusTopicPublisher {
    pub(crate) inner: RustTopicPublisher,
}

pub(crate) struct RobotBusServiceClient {
    pub(crate) inner: RustNodeServiceClient,
}

pub(crate) struct RobotBusActionClient {
    pub(crate) inner: RustNodeActionClient,
}

pub(crate) struct RobotBusActionGoalHandle {
    inner: RustActionGoalHandle,
    feedback: Option<
        Arc<
            Mutex<
                Option<(
                    unsafe extern "C" fn(*const RobotBusActionMessage, *mut c_void),
                    usize,
                )>,
            >,
        >,
    >,
}

#[repr(C)]
pub(crate) struct RobotBusActionMessage {
    pub kind: *mut c_char,
    pub body: *mut u8,
    pub body_len: usize,
    pub goal_id: *mut c_char,
    pub action_name: *mut c_char,
}

fn action_kind_str(k: ActionKind) -> &'static str {
    match k {
        ActionKind::Goal => "GOAL",
        ActionKind::Feedback => "FEEDBACK",
        ActionKind::Result => "RESULT",
        ActionKind::Cancel => "CANCEL",
    }
}

fn free_action_message(m: &RobotBusActionMessage) {
    robot_bus_free_string(m.kind);
    robot_bus_free_bytes(m.body, m.body_len);
    robot_bus_free_string(m.goal_id);
    robot_bus_free_string(m.action_name);
}

fn owned_action_message(message: &robot_bus::action_bus::ActionMessage) -> RobotBusActionMessage {
    let (body, body_len) = dup_bytes(&message.body);
    RobotBusActionMessage {
        kind: dup_string(action_kind_str(message.kind)),
        body,
        body_len,
        goal_id: dup_string(&message.goal_id),
        action_name: dup_string(&message.action_name),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_messages_free(msgs: *mut RobotBusActionMessage, count: usize) {
    if msgs.is_null() || count == 0 {
        return;
    }
    unsafe {
        let slice = slice::from_raw_parts_mut(msgs, count);
        for m in slice.iter_mut() {
            free_action_message(m);
        }
        drop(Vec::from_raw_parts(msgs, count, count));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_topic_publisher_free(p: *mut RobotBusTopicPublisher) {
    if !p.is_null() {
        unsafe {
            drop(Box::from_raw(p));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_topic_publisher_topic(p: *const RobotBusTopicPublisher) -> *mut c_char {
    if p.is_null() {
        set_error("null topic publisher");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*p }.inner.topic())
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_topic_publisher_publish(
    p: *mut RobotBusTopicPublisher,
    data: *const u8,
    len: usize,
) -> c_int {
    if p.is_null() {
        return err("null topic publisher");
    }
    let bytes = match bytes_slice(data, len) {
        Ok(b) => b,
        Err(e) => return e,
    };
    match unsafe { &*p }.inner.publish(bytes) {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_service_client_free(c: *mut RobotBusServiceClient) {
    if !c.is_null() {
        unsafe {
            drop(Box::from_raw(c));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_service_client_service_name(
    c: *const RobotBusServiceClient,
) -> *mut c_char {
    if c.is_null() {
        set_error("null service client");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*c }.inner.service_name())
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_service_client_call(
    c: *mut RobotBusServiceClient,
    data: *const u8,
    len: usize,
    timeout_secs: f64,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if c.is_null() || out_data.is_null() || out_len.is_null() {
        return err("null argument");
    }
    let bytes = match bytes_slice(data, len) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(timeout_secs))
    };
    match unsafe { &*c }.inner.call(bytes, timeout) {
        Ok(reply) => {
            let (ptr, n) = dup_bytes(&reply);
            unsafe {
                *out_data = ptr;
                *out_len = n;
            }
            ok()
        }
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_client_free(c: *mut RobotBusActionClient) {
    if !c.is_null() {
        unsafe {
            drop(Box::from_raw(c));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_client_action_name(
    c: *const RobotBusActionClient,
) -> *mut c_char {
    if c.is_null() {
        set_error("null action client");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*c }.inner.action_name())
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_client_send_goal(
    c: *mut RobotBusActionClient,
    data: *const u8,
    len: usize,
    goal_id: *const c_char,
    timeout_secs: f64,
    feedback: RobotBusActionFeedbackCallback,
    user: *mut c_void,
    out_handle: *mut *mut RobotBusActionGoalHandle,
) -> c_int {
    if c.is_null() || out_handle.is_null() {
        return err("null argument");
    }
    let bytes = match bytes_slice(data, len) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let goal_id = cstr_opt(goal_id);
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(timeout_secs))
    };
    let feedback_state =
        feedback.map(|callback| Arc::new(Mutex::new(Some((callback, user as usize)))));
    let feedback_callback: Option<RawActionFeedbackCallback> =
        feedback_state.as_ref().map(|state| {
            let state = Arc::clone(state);
            Arc::new(move |message: &robot_bus::action_bus::ActionMessage| {
                let kind = CString::new(action_kind_str(message.kind)).expect("static action kind");
                let goal_id = CString::new(message.goal_id.as_str()).unwrap_or_default();
                let action_name = CString::new(message.action_name.as_str()).unwrap_or_default();
                let borrowed = RobotBusActionMessage {
                    kind: kind.as_ptr() as *mut c_char,
                    body: message.body.as_ptr() as *mut u8,
                    body_len: message.body.len(),
                    goal_id: goal_id.as_ptr() as *mut c_char,
                    action_name: action_name.as_ptr() as *mut c_char,
                };
                if let Ok(target) = state.lock() {
                    if let Some((callback, user)) = *target {
                        unsafe { callback(&borrowed, user as *mut c_void) };
                    }
                }
            }) as RawActionFeedbackCallback
        });
    match unsafe { &*c }
        .inner
        .send_goal(bytes, goal_id, timeout, feedback_callback)
    {
        Ok(inner) => {
            unsafe {
                *out_handle = Box::into_raw(Box::new(RobotBusActionGoalHandle {
                    inner,
                    feedback: feedback_state,
                }));
            }
            ok()
        }
        Err(e) => bus_err(e),
    }
}

pub type RobotBusActionFeedbackCallback =
    Option<unsafe extern "C" fn(message: *const RobotBusActionMessage, user: *mut c_void)>;

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_goal_handle_free(h: *mut RobotBusActionGoalHandle) {
    if !h.is_null() {
        unsafe {
            let handle = Box::from_raw(h);
            if let Some(feedback) = &handle.feedback {
                if let Ok(mut target) = feedback.lock() {
                    target.take();
                }
            }
            drop(handle);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_goal_handle_goal_id(
    h: *const RobotBusActionGoalHandle,
) -> *mut c_char {
    if h.is_null() {
        set_error("null action goal handle");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*h }.inner.goal_id())
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_goal_handle_action_name(
    h: *const RobotBusActionGoalHandle,
) -> *mut c_char {
    if h.is_null() {
        set_error("null action goal handle");
        return ptr::null_mut();
    }
    clear_error();
    dup_string(unsafe { &*h }.inner.action_name())
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_goal_handle_wait_result(
    h: *mut RobotBusActionGoalHandle,
    timeout_secs: f64,
    out_msg: *mut RobotBusActionMessage,
) -> c_int {
    if h.is_null() || out_msg.is_null() {
        return err("null argument");
    }
    let handle = unsafe { &*h }.inner.clone();
    let result = if timeout_secs < 0.0 {
        handle.wait_result()
    } else {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let _ = tx.send(handle.wait_result());
        });
        rx.recv_timeout(Duration::from_secs_f64(timeout_secs))
            .map_err(|_| {
                robot_bus::errors::BusError::Timeout(format!(
                    "action result timed out after {timeout_secs}s"
                ))
            })
            .and_then(|result| result)
    };
    match result {
        Ok(message) => {
            unsafe {
                *out_msg = owned_action_message(&message);
            }
            ok()
        }
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_goal_handle_cancel(h: *mut RobotBusActionGoalHandle) -> c_int {
    if h.is_null() {
        return err("null action goal handle");
    }
    match unsafe { &*h }.inner.cancel() {
        Ok(()) => ok(),
        Err(e) => bus_err(e),
    }
}

// --- Callbacks --------------------------------------------------------------

pub type RobotBusMsgCallback = Option<
    unsafe extern "C" fn(topic: *const c_char, data: *const u8, len: usize, user: *mut c_void),
>;
pub type RobotBusTimerCallback = Option<unsafe extern "C" fn(user: *mut c_void)>;
pub type RobotBusServiceHandler = Option<
    unsafe extern "C" fn(
        data: *const u8,
        len: usize,
        out_data: *mut *mut u8,
        out_len: *mut usize,
        user: *mut c_void,
    ) -> c_int,
>;
pub type RobotBusActionHandler = Option<
    unsafe extern "C" fn(
        data: *const u8,
        len: usize,
        out_phases: *mut *mut RobotBusActionPhase,
        out_count: *mut usize,
        user: *mut c_void,
    ) -> c_int,
>;

#[repr(C)]
pub(crate) struct RobotBusActionPhase {
    pub phase: *mut c_char,
    pub body: *mut u8,
    pub body_len: usize,
}

/// Allocate `count` zeroed [`RobotBusActionPhase`] slots (pair with handler return /
/// [`robot_bus_action_phases_free`]).
#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_alloc_action_phases(count: usize) -> *mut RobotBusActionPhase {
    if count == 0 {
        return ptr::null_mut();
    }
    let mut v = Vec::with_capacity(count);
    for _ in 0..count {
        v.push(RobotBusActionPhase {
            phase: ptr::null_mut(),
            body: ptr::null_mut(),
            body_len: 0,
        });
    }
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_phases_free(phases: *mut RobotBusActionPhase, count: usize) {
    if phases.is_null() || count == 0 {
        return;
    }
    unsafe {
        let slice = slice::from_raw_parts_mut(phases, count);
        for p in slice.iter_mut() {
            robot_bus_free_string(p.phase);
            robot_bus_free_bytes(p.body, p.body_len);
        }
        drop(Vec::from_raw_parts(phases, count, count));
    }
}
