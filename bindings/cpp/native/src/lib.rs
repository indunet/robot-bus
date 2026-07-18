//! C ABI for robot-bus (opaque handles). Mirrored from Python / napi surfaces.

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::time::Duration;

use robot_bus::action_bus::ActionKind;
use robot_bus::broker::{
    RobotBusBroker as RustRobotBusBroker, RobotBusConfig,
};
use robot_bus::errors::BusError;
use robot_bus::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use robot_bus::runtime::{
    ActionGoalHandler, CallbackGroup, CallbackGroupType,
    MultiThreadedExecutor as RustMultiThreadedExecutor, Node as RustNode,
    NodeActionClientRaw as RustNodeActionClient, NodeOptions as RustNodeOptions,
    NodeServiceClientRaw as RustNodeServiceClient, ServiceHandler,
    ShutdownHandle as RustShutdownHandle, SingleThreadedExecutor as RustSingleThreadedExecutor,
    TimerCallback, TimerHandle as RustTimerHandle, TopicPublisherRaw as RustTopicPublisher,
};
use robot_bus::transports;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_error(msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    let c = CString::new(msg.replace('\0', "")).unwrap_or_else(|_| CString::new("error").unwrap());
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(c));
}

fn clear_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
}

fn ok() -> c_int {
    clear_error();
    0
}

fn err(msg: impl AsRef<str>) -> c_int {
    set_error(msg);
    -1
}

fn bus_err(e: BusError) -> c_int {
    err(e.to_string())
}

fn anyhow_err(e: anyhow::Error) -> c_int {
    err(e.to_string())
}

fn cstr_opt<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(p) }.to_str().ok()
    }
}

fn cstr_req<'a>(p: *const c_char) -> Result<&'a str, c_int> {
    cstr_opt(p).ok_or_else(|| err("null string"))
}

fn bytes_slice<'a>(data: *const u8, len: usize) -> Result<&'a [u8], c_int> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(err("null bytes"));
    }
    Ok(unsafe { slice::from_raw_parts(data, len) })
}

fn normalize_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
    }
}

fn node_options(
    host: &str,
    transport: &str,
    grpc_url: Option<String>,
    message_xsub: Option<String>,
    message_xpub: Option<String>,
    service_frontend: Option<String>,
    service_backend: Option<String>,
    action_backend: Option<String>,
    action_frontend: Option<String>,
) -> Result<RustNodeOptions, c_int> {
    if transport == "grpc" {
        return Ok(match grpc_url {
            Some(url) => RustNodeOptions::grpc_at(url),
            None => RustNodeOptions::grpc(),
        });
    }
    if grpc_url.is_some() {
        return Err(err("grpc_url is only valid when transport=\"grpc\""));
    }
    Ok(RustNodeOptions {
        host: host.into(),
        transport: transport.into(),
        grpc_url: None,
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

fn dup_string(s: &str) -> *mut c_char {
    CString::new(s.replace('\0', ""))
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

fn dup_bytes(b: &[u8]) -> (*mut u8, usize) {
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

// --- Publisher / Subscriber -------------------------------------------------

pub struct RobotBusPublisher {
    inner: RustPublisher,
}

pub struct RobotBusSubscriber {
    inner: RustSubscriber,
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

// --- Shutdown / Timer / CallbackGroup ---------------------------------------

pub struct RobotBusShutdownHandle {
    inner: RustShutdownHandle,
}

pub struct RobotBusTimerHandle {
    inner: RustTimerHandle,
}

pub struct RobotBusCallbackGroup {
    inner: CallbackGroup,
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

pub struct RobotBusTopicPublisher {
    inner: RustTopicPublisher,
}

pub struct RobotBusServiceClient {
    inner: RustNodeServiceClient,
}

pub struct RobotBusActionClient {
    inner: RustNodeActionClient,
}

#[repr(C)]
pub struct RobotBusActionMessage {
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
    out_msgs: *mut *mut RobotBusActionMessage,
    out_count: *mut usize,
) -> c_int {
    if c.is_null() || out_msgs.is_null() || out_count.is_null() {
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
    match unsafe { &*c }.inner.send_goal(bytes, goal_id, timeout) {
        Ok(messages) => {
            let mut out: Vec<RobotBusActionMessage> = messages
                .into_iter()
                .map(|m| {
                    let (body, body_len) = dup_bytes(&m.body);
                    RobotBusActionMessage {
                        kind: dup_string(action_kind_str(m.kind)),
                        body,
                        body_len,
                        goal_id: dup_string(&m.goal_id),
                        action_name: dup_string(&m.action_name),
                    }
                })
                .collect();
            let count = out.len();
            let ptr = out.as_mut_ptr();
            std::mem::forget(out);
            unsafe {
                *out_msgs = ptr;
                *out_count = count;
            }
            ok()
        }
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_action_client_cancel(
    c: *mut RobotBusActionClient,
    goal_id: *const c_char,
    data: *const u8,
    len: usize,
    timeout_secs: f64,
    out_msg: *mut RobotBusActionMessage,
) -> c_int {
    if c.is_null() || out_msg.is_null() {
        return err("null argument");
    }
    let goal_id = match cstr_req(goal_id) {
        Ok(g) => g,
        Err(e) => return e,
    };
    let bytes = match bytes_slice(data, len) {
        Ok(b) => b,
        Err(e) => return e,
    };
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(Duration::from_secs_f64(timeout_secs))
    };
    match unsafe { &*c }.inner.cancel(goal_id, bytes, timeout) {
        Ok(m) => {
            let (body, body_len) = dup_bytes(&m.body);
            unsafe {
                *out_msg = RobotBusActionMessage {
                    kind: dup_string(action_kind_str(m.kind)),
                    body,
                    body_len,
                    goal_id: dup_string(&m.goal_id),
                    action_name: dup_string(&m.action_name),
                };
            }
            ok()
        }
        Err(e) => bus_err(e),
    }
}

// --- Callbacks --------------------------------------------------------------

pub type RobotBusMsgCallback =
    Option<unsafe extern "C" fn(topic: *const c_char, data: *const u8, len: usize, user: *mut c_void)>;
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
pub struct RobotBusActionPhase {
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

// --- Node -------------------------------------------------------------------

pub struct RobotBusNode {
    inner: RustNode,
}

#[repr(C)]
pub struct RobotBusNodeOptions {
    pub host: *const c_char,
    pub transport: *const c_char,
    pub grpc_url: *const c_char,
    pub message_xsub: *const c_char,
    pub message_xpub: *const c_char,
    pub service_frontend: *const c_char,
    pub service_backend: *const c_char,
    pub action_backend: *const c_char,
    pub action_frontend: *const c_char,
}

fn parse_node_options(opts: *const RobotBusNodeOptions) -> Result<RustNodeOptions, c_int> {
    if opts.is_null() {
        return Ok(RustNodeOptions::default());
    }
    let o = unsafe { &*opts };
    let host = cstr_opt(o.host).unwrap_or("localhost");
    let transport = cstr_opt(o.transport).unwrap_or("tcp");
    node_options(
        host,
        transport,
        cstr_opt(o.grpc_url).map(str::to_string),
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
pub extern "C" fn robot_bus_node_grpc(name: *const c_char) -> *mut RobotBusNode {
    clear_error();
    let name = match cstr_req(name) {
        Ok(n) => n.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    Box::into_raw(Box::new(RobotBusNode {
        inner: RustNode::grpc(name),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_grpc_at(
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
        inner: RustNode::grpc_at(name, url),
    }))
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
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let topic = match cstr_req(topic) {
        Ok(t) => t,
        Err(_) => return ptr::null_mut(),
    };
    match unsafe { &mut *n }.inner.create_publisher_raw(topic) {
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
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    let Some(cb_fn) = callback else {
        return err("null callback");
    };
    let topic = match cstr_req(topic) {
        Ok(t) => t,
        Err(e) => return e,
    };
    // user pointer is assumed to outlive the subscription (caller responsibility).
    let user = user as usize;
    let cb: robot_bus::runtime::MessageCallback = Arc::new(move |topic, payload| {
        let c_topic = CString::new(topic.replace('\0', "")).unwrap_or_default();
        unsafe {
            cb_fn(
                c_topic.as_ptr(),
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
    match unsafe { &mut *n }
        .inner
        .create_subscription_raw(topic, cb, group)
    {
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
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    let Some(handler_fn) = handler else {
        return err("null handler");
    };
    let service_name = match cstr_req(service_name) {
        Ok(s) => s,
        Err(e) => return e,
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
    match unsafe { &mut *n }
        .inner
        .create_service_raw(service_name, cb, group)
    {
        Ok(_) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_client(
    n: *mut RobotBusNode,
    service_name: *const c_char,
) -> *mut RobotBusServiceClient {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let service_name = match cstr_req(service_name) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match unsafe { &mut *n }.inner.create_client_raw(service_name) {
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
) -> c_int {
    if n.is_null() {
        return err("null node");
    }
    let Some(handler_fn) = handler else {
        return err("null handler");
    };
    let action_name = match cstr_req(action_name) {
        Ok(s) => s,
        Err(e) => return e,
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
    match unsafe { &mut *n }
        .inner
        .create_action_server_raw(action_name, cb, group)
    {
        Ok(_) => ok(),
        Err(e) => bus_err(e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_node_create_action_client(
    n: *mut RobotBusNode,
    action_name: *const c_char,
) -> *mut RobotBusActionClient {
    if n.is_null() {
        set_error("null node");
        return ptr::null_mut();
    }
    let action_name = match cstr_req(action_name) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match unsafe { &mut *n }.inner.create_action_client_raw(action_name) {
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

// --- Executors --------------------------------------------------------------

pub struct RobotBusSingleThreadedExecutor {
    inner: RustSingleThreadedExecutor,
}

pub struct RobotBusMultiThreadedExecutor {
    inner: RustMultiThreadedExecutor,
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_single_threaded_executor_new() -> *mut RobotBusSingleThreadedExecutor {
    clear_error();
    Box::into_raw(Box::new(RobotBusSingleThreadedExecutor {
        inner: RustSingleThreadedExecutor::new(),
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

pub struct RobotBusBroker {
    inner: Option<RustRobotBusBroker>,
}

#[repr(C)]
pub struct RobotBusBrokerOptions {
    pub message_xsub_bind: *const c_char,
    pub message_xpub_bind: *const c_char,
    pub service_frontend_bind: *const c_char,
    pub service_backend_bind: *const c_char,
    pub action_frontend_bind: *const c_char,
    pub action_backend_bind: *const c_char,
    pub grpc_listen: *const c_char,
    pub console_listen: *const c_char,
    pub tcp_only: c_int,
    pub no_console: c_int,
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_start(
    opts: *const RobotBusBrokerOptions,
) -> *mut RobotBusBroker {
    clear_error();
    let mut config = RobotBusConfig::default();
    if !opts.is_null() {
        let o = unsafe { &*opts };
        if let Some(v) = cstr_opt(o.message_xsub_bind) {
            config.message.xsub_bind = normalize_bind(v);
        }
        if let Some(v) = cstr_opt(o.message_xpub_bind) {
            config.message.xpub_bind = normalize_bind(v);
        }
        if let Some(v) = cstr_opt(o.service_frontend_bind) {
            config.service.frontend_bind = normalize_bind(v);
        }
        if let Some(v) = cstr_opt(o.service_backend_bind) {
            config.service.backend_bind = normalize_bind(v);
        }
        if let Some(v) = cstr_opt(o.action_frontend_bind) {
            config.action.frontend_bind = normalize_bind(v);
        }
        if let Some(v) = cstr_opt(o.action_backend_bind) {
            config.action.backend_bind = normalize_bind(v);
        }
        if o.tcp_only != 0 {
            config.message.bind_all_transports = false;
            config.service.bind_all_transports = false;
            config.action.bind_all_transports = false;
        }
        if let Some(v) = cstr_opt(o.grpc_listen) {
            match v.parse() {
                Ok(addr) => config.grpc.listen = addr,
                Err(e) => {
                    set_error(format!("invalid grpc_listen: {e}"));
                    return ptr::null_mut();
                }
            }
        }
        if o.no_console != 0 {
            config.console.enabled = false;
        }
        if let Some(v) = cstr_opt(o.console_listen) {
            match v.parse() {
                Ok(addr) => {
                    config.console.listen = addr;
                    config.console.enabled = true;
                }
                Err(e) => {
                    set_error(format!("invalid console_listen: {e}"));
                    return ptr::null_mut();
                }
            }
        }
    }
    match RustRobotBusBroker::start(config) {
        Ok(inner) => Box::into_raw(Box::new(RobotBusBroker {
            inner: Some(inner),
        })),
        Err(e) => {
            set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_free(b: *mut RobotBusBroker) {
    if b.is_null() {
        return;
    }
    unsafe {
        let mut broker = Box::from_raw(b);
        if let Some(inner) = broker.inner.take() {
            let _ = inner.stop();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_stop(b: *mut RobotBusBroker) -> c_int {
    if b.is_null() {
        return err("null broker");
    }
    let broker = unsafe { &mut *b };
    if let Some(inner) = broker.inner.take() {
        match inner.stop() {
            Ok(()) => ok(),
            Err(e) => anyhow_err(e),
        }
    } else {
        ok()
    }
}

fn broker_ref(b: *const RobotBusBroker) -> Result<&'static RustRobotBusBroker, c_int> {
    if b.is_null() {
        return Err(err("null broker"));
    }
    match unsafe { &*b }.inner.as_ref() {
        Some(inner) => Ok(unsafe {
            // Lifetime is tied to the broker handle; caller must keep it alive.
            &*(inner as *const RustRobotBusBroker)
        }),
        None => Err(err("broker already stopped")),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_message_xsub_bind(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.message.xsub_bind)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_message_xpub_bind(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.message.xpub_bind)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_service_frontend_bind(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.service.frontend_bind)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_service_backend_bind(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.service.backend_bind)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_action_frontend_bind(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.action.frontend_bind)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_action_backend_bind(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.action.backend_bind)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_grpc_listen(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.grpc_listen().to_string())
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_console_listen(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            match inner.console_listen() {
                Some(addr) => dup_string(&addr.to_string()),
                None => ptr::null_mut(),
            }
        }
        Err(_) => ptr::null_mut(),
    }
}
