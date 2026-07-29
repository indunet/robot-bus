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
    apply_federation_opts, RobotBusBroker as RustRobotBusBroker, RobotBusConfig,
};
use robot_bus::discovery::{
    wait as discover_wait, DiscoverOpts as RustDiscoverOpts, DEFAULT_DISCOVERY_PORT,
    DEFAULT_DISCOVERY_TIMEOUT, DEFAULT_MULTICAST_ADDR,
};
use robot_bus::errors::BusError;
use robot_bus::message_bus::{Publisher as RustPublisher, Subscriber as RustSubscriber};
use robot_bus::runtime::{
    ActionGoalHandler, CallbackGroup, CallbackGroupType, Context as RustContext,
    MultiThreadedExecutor as RustMultiThreadedExecutor, Node as RustNode,
    NodeActionClientRaw as RustNodeActionClient, NodeOptions as RustNodeOptions,
    NodeServiceClientRaw as RustNodeServiceClient, ParameterValue, ServiceHandler,
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

// --- Context ----------------------------------------------------------------

pub struct RobotBusContext {
    inner: RustContext,
}

fn context_ref(ctx: *mut RobotBusContext) -> Result<&'static RustContext, c_int> {
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
pub extern "C" fn robot_bus_node_new_with_context(
    ctx: *mut RobotBusContext,
    name: *const c_char,
    opts: *const RobotBusNodeOptions,
) -> *mut RobotBusNode {
    clear_error();
    let context = match context_ref(ctx) {
        Ok(c) => c.clone(),
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
        inner: RustNode::with_context(context, name, options),
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
        Ok(c) => c.clone(),
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

#[repr(C)]
pub struct RobotBusDiscoverOpts {
    pub domain_id: u32,
    pub broker_id: *const c_char,
    pub multicast_addr: *const c_char,
    pub multicast_port: u16,
    pub timeout_secs: f64,
}

#[repr(C)]
pub struct RobotBusAppliedNodeOptions {
    pub host: *mut c_char,
    pub transport: *mut c_char,
    pub grpc_url: *mut c_char,
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
    out.domain_id = o.domain_id;
    if let Some(id) = cstr_opt(o.broker_id) {
        if !id.is_empty() {
            out.broker_id = Some(id.to_string());
        }
    }
    if let Some(addr) = cstr_opt(o.multicast_addr) {
        if !addr.is_empty() {
            out.multicast_addr = addr.parse().map_err(|e| {
                set_error(format!("invalid multicast_addr: {e}"));
            })?;
        }
    }
    if o.multicast_port != 0 {
        out.multicast_port = o.multicast_port;
    }
    if o.timeout_secs > 0.0 {
        out.timeout = Duration::from_secs_f64(o.timeout_secs);
    }
    let _ = (DEFAULT_MULTICAST_ADDR, DEFAULT_DISCOVERY_PORT, DEFAULT_DISCOVERY_TIMEOUT);
    Ok(out)
}

fn transport_base_options(transport: &str) -> Result<RustNodeOptions, c_int> {
    match transport {
        "tcp" => Ok(RustNodeOptions::tcp()),
        "ipc" => Ok(RustNodeOptions::ipc()),
        "inproc" => Ok(RustNodeOptions::inproc()),
        "grpc" => Ok(RustNodeOptions::grpc()),
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
        (*out).grpc_url = options
            .grpc_url
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
        robot_bus_free_string((*o).grpc_url);
        robot_bus_free_string((*o).message_xsub);
        robot_bus_free_string((*o).message_xpub);
        robot_bus_free_string((*o).service_frontend);
        robot_bus_free_string((*o).service_backend);
        robot_bus_free_string((*o).action_backend);
        robot_bus_free_string((*o).action_frontend);
        (*o).host = ptr::null_mut();
        (*o).transport = ptr::null_mut();
        (*o).grpc_url = ptr::null_mut();
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

#[repr(C)]
pub struct RobotBusParameterValue {
    pub type_: c_int,
    pub bool_value: c_int,
    pub integer_value: i64,
    pub double_value: f64,
    pub string_value: *mut c_char,
}

#[repr(C)]
pub struct RobotBusParameter {
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
        Ok(()) => ok(),
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
    match unsafe { &mut *n }.inner.set_parameter(name, pv) {
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
        Ok(v) => {
            unsafe { *out = parameter_value_to_c(v) };
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
    let list = unsafe { &*n }.inner.list_parameters();
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
    /// Hop-path id for federation (nullable / empty → random UUID at start).
    pub broker_id: *const c_char,
    /// Peer XPUB endpoints (`MessagePeer::from_xpub`); length `message_peer_count`.
    pub message_peers: *const *const c_char,
    pub message_peer_count: usize,
    /// Peer service backends (`ServicePeer::from_backend`); length `service_peer_count`.
    pub service_peers: *const *const c_char,
    pub service_peer_count: usize,
    /// Peer action backends (`ActionPeer::from_backend`); length `action_peer_count`.
    pub action_peers: *const *const c_char,
    pub action_peer_count: usize,
    pub no_discovery: c_int,
    pub domain_id: u32,
    pub advertise_host: *const c_char,
    pub discovery_addr: *const c_char,
    pub discovery_port: u16,
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_start(
    opts: *const RobotBusBrokerOptions,
) -> *mut RobotBusBroker {
    robot_bus_broker_start_with_context(ptr::null_mut(), opts)
}

fn cstr_array(ptrs: *const *const c_char, count: usize) -> Result<Vec<String>, ()> {
    if count == 0 || ptrs.is_null() {
        return Ok(Vec::new());
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let p = unsafe { *ptrs.add(i) };
        match cstr_opt(p) {
            Some(s) => out.push(s.to_string()),
            None => {
                set_error("null peer string");
                return Err(());
            }
        }
    }
    Ok(out)
}

fn parse_broker_config(opts: *const RobotBusBrokerOptions) -> Result<RobotBusConfig, ()> {
    let mut config = RobotBusConfig::default();
    if opts.is_null() {
        return Ok(config);
    }
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
                return Err(());
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
                return Err(());
            }
        }
    }

    let message_peers = cstr_array(o.message_peers, o.message_peer_count)?;
    let service_peers = cstr_array(o.service_peers, o.service_peer_count)?;
    let action_peers = cstr_array(o.action_peers, o.action_peer_count)?;
    if let Err(e) = apply_federation_opts(
        &mut config,
        cstr_opt(o.broker_id),
        &message_peers,
        &service_peers,
        &action_peers,
    ) {
        set_error(e.to_string());
        return Err(());
    }

    if o.no_discovery != 0 {
        config.discovery.enabled = false;
    }
    config.discovery.domain_id = o.domain_id;
    if let Some(v) = cstr_opt(o.advertise_host) {
        if !v.is_empty() {
            config.discovery.advertise_host = Some(v.to_string());
        }
    }
    if let Some(v) = cstr_opt(o.discovery_addr) {
        if !v.is_empty() {
            config.discovery.multicast_addr = v.parse().map_err(|e| {
                set_error(format!("invalid discovery_addr: {e}"));
            })?;
        }
    }
    if o.discovery_port != 0 {
        config.discovery.multicast_port = o.discovery_port;
    }

    Ok(config)
}

#[unsafe(no_mangle)]
pub extern "C" fn robot_bus_broker_start_with_context(
    ctx: *mut RobotBusContext,
    opts: *const RobotBusBrokerOptions,
) -> *mut RobotBusBroker {
    clear_error();
    let config = match parse_broker_config(opts) {
        Ok(c) => c,
        Err(()) => return ptr::null_mut(),
    };
    let started = if ctx.is_null() {
        RustRobotBusBroker::start(config)
    } else {
        match context_ref(ctx) {
            Ok(c) => RustRobotBusBroker::start_with_context(c.clone(), config),
            Err(_) => return ptr::null_mut(),
        }
    };
    match started {
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
