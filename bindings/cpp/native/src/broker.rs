//! RobotBusBroker C ABI.

use std::os::raw::{c_char, c_int};
use std::ptr;

use robot_bus::broker::{
    apply_federation_opts, RobotBusBroker as RustRobotBusBroker, RobotBusConfig,
};

use crate::context::{context_ref, RobotBusContext};
use crate::ffi::{
    anyhow_err, clear_error, cstr_opt, dup_string, err, normalize_bind, ok, set_error,
};

pub(crate) struct RobotBusBroker {
    pub(crate) inner: Option<RustRobotBusBroker>,
}

#[repr(C)]
pub(crate) struct RobotBusBrokerOptions {
    pub message_xsub_bind: *const c_char,
    pub message_xpub_bind: *const c_char,
    pub service_frontend_bind: *const c_char,
    pub service_backend_bind: *const c_char,
    pub action_frontend_bind: *const c_char,
    pub action_backend_bind: *const c_char,
    pub api_listen: *const c_char,
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
    /// Federation peers as API URLs / `host:port` (GET /api/v1/discover).
    pub peers: *const *const c_char,
    pub peer_count: usize,
    /// When non-zero, hide tank demo and reject tank session acquire.
    pub no_tank: c_int,
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
    if let Some(v) = cstr_opt(o.api_listen) {
        match v.parse() {
            Ok(addr) => config.ws.listen = addr,
            Err(e) => {
                set_error(format!("invalid api_listen: {e}"));
                return Err(());
            }
        }
    }
    if o.no_console != 0 {
        config.console.enabled = false;
    }
    if o.no_tank != 0 {
        config.console.tank_enabled = false;
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
    let api = cstr_opt(o.api_listen);
    if let Some(v) = api {
        if !v.is_empty() {
            config.ws.listen = v.parse().map_err(|e| {
                set_error(format!("invalid api_listen: {e}"));
            })?;
            config.console.listen = config.ws.listen;
        }
    }
    let peers = cstr_array(o.peers, o.peer_count)?;
    if let Err(e) = robot_bus::apply_api_peers(&mut config, &peers) {
        set_error(e.to_string());
        return Err(());
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
pub extern "C" fn robot_bus_broker_api_listen(b: *const RobotBusBroker) -> *mut c_char {
    match broker_ref(b) {
        Ok(inner) => {
            clear_error();
            dup_string(&inner.api_listen().to_string())
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

